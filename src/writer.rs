use std::sync::{Arc,Mutex};
use std::io::{Write,BufWriter,Result};
use std::collections::BTreeMap;

use ulid::*;
use bzip2::Compression;
use bzip2::write::BzEncoder;
use mysql::*;
use mysql::prelude::*;
use super::gcs::*;

use super::formatter::*;
use super::selfplay::*;
use super::logic::Setting;

////////////////////////////////////////////////////////////////////////////////
// Trait
////////////////////////////////////////////////////////////////////////////////

pub trait WriteRecord {
    fn write_record(&mut self, record:Record) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

////////////////////////////////////////////////////////////////////////////////
// Evaluator
////////////////////////////////////////////////////////////////////////////////

pub struct EvaluationWriter {
    mysql_pool : Arc<Mutex<Pool>>,
    plays_per_write : usize,
    buffer : Vec<Record>,
}

impl EvaluationWriter {
    pub fn new( mysql_pool:Arc<Mutex<Pool>>, plays_per_write:usize ) -> EvaluationWriter {
        EvaluationWriter {
            mysql_pool : mysql_pool,
            plays_per_write : plays_per_write,
            buffer : vec!{},
        }
    }
}

// 戻り値はデッドロック回避のためにHashMapではなくBTreeMapである必要があります。
//
// MySQLでINSERTのデッドロックに嵌る人を1人でも減らすために
// https://ichirin2501.hatenablog.com/entry/2015/12/24/164916
//
// A> INSERT INTO player (name) VALUES("a"),("b"),("c"),....,("z");
// B> INSERT INTO player (name) VALUES("z"),("y"),("x"),....,("a");
//
// 上記クエリが同時実行されるとdeadlockが発生する可能性がある、
// という説明で問題がだいたい理解できると思います。
// ロックを纏めて取るのではなく行ロックで１個ずつ確保してしまうから、
// どこかでデッドロックしてしまうわけです。
fn aggregate_records( records:&Vec<Record> ) -> BTreeMap<String,(f64,usize)> {
    let mut ret = BTreeMap::new();

    for record in records {
        let (reward,count) = ret.entry(record.name.clone()).or_insert((0.0,0));
        *reward += record.reward as f64;
        *count += 1;
    }

    return ret;
}

fn write_record_flush_buffer( mysql_pool:&Arc<Mutex<Pool>>, buf:&Vec<Record> ) {
    // リプレイデータの打ち上げ
    {
        let encoded: Vec<u8> = bincode::serialize(&buf).unwrap();

        {
            let file = std::fs::File::create("record.bincode.bz2").unwrap();
            let mut writer = BzEncoder::new(BufWriter::new(file), Compression::best());
            writer.write_all(&encoded).unwrap();
        }

        // アップロードするファイル名を決定します
        let ulid = Ulid::new().to_string();

        // ファイルの打ち上げ
        eprintln!("{} Uploading...", ulid);
        let destination_path = format!("record/{}.bz2", ulid);
        match upload("record.bincode.bz2",&destination_path,"application/x-bzip2") {
            Ok(()) => eprintln!("{} Done.", ulid),
            Err(x) => eprintln!("{} {}", ulid, x),
        }
    }

    // mysqlに評価の書き込み
    {
        let mut conn = mysql_pool.lock().unwrap().get_conn().unwrap();
        let mut tx = conn.start_transaction(TxOpts::default()).unwrap();

        let sum = aggregate_records(&buf);

        eprintln!("Update evaluations... {:?}", sum);

        tx.exec_batch(
            "INSERT INTO evaluation (name, total_reward, total_count) VALUES (:name, :reward, :count) \
            ON DUPLICATE KEY UPDATE total_reward=total_reward+VALUES(total_reward), total_count=total_count+VALUES(total_count)",
            sum.iter().map(|(k,(reward,count))| params! {"name" => k.clone(), "reward" => reward, "count" => count})
        ).unwrap();

        tx.exec_batch(
            "INSERT INTO episode (name, reward, quality, turn) VALUES (:name, :reward, :quality, :turn)",
            buf.iter().map(|x| params! {"name" => x.name.clone(), "reward" => x.reward, "quality" => x.last_state.quality, "turn" => x.last_state.turn - 1 })
        ).unwrap();

        tx.commit().unwrap();
    }
}

impl WriteRecord for EvaluationWriter {
    fn write_record(&mut self, record:Record) -> Result<()> {
        self.buffer.push(record);

        if self.buffer.len() >= self.plays_per_write {
            write_record_flush_buffer( &self.mysql_pool, &self.buffer );
            self.buffer.clear();
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if self.buffer.len() > 0 {
            write_record_flush_buffer( &self.mysql_pool, &self.buffer );
            self.buffer.clear();
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Generator
////////////////////////////////////////////////////////////////////////////////

pub struct GenerationWriter {
    mysql_pool : Arc<Mutex<Pool>>,
    setting : Setting,
    plays_per_write : usize,
    buffer : Vec<Record>,
}

impl GenerationWriter {
    pub fn new( mysql_pool:Arc<Mutex<Pool>>, plays_per_write:usize, setting:Setting ) -> GenerationWriter {
        GenerationWriter {
            mysql_pool : mysql_pool,
            setting : setting,
            plays_per_write : plays_per_write,
            buffer : vec!{},
        }
    }
}

fn write_samples<W:Write,F:Formatter>( writer:&mut W, formatter:&F, record:&Record) -> Result<()> {
    for x in formatter.format(&record) {
        writer.write_all(x.as_bytes())?;
        writer.write_all(&['\n' as u8])?;
    }

    Ok(())
}

fn write_samples_flush_buffer( mysql_pool:&Arc<Mutex<Pool>>, setting:&Setting, buf:&Vec<Record> ) {

    // アップロードするファイル名を決定します
    let ulid = Ulid::new().to_string();

    // ファイルに全部書き込み
    eprintln!("{} Output records...", ulid);

    {
        let file = std::fs::File::create("sample.txt.bz2").unwrap();
        let mut writer = BzEncoder::new(BufWriter::new(file), Compression::best());
        let formatter = TsvFormatter { setting:setting.clone()};

        for x in buf {
            write_samples( &mut writer, &formatter, x ).unwrap()
        }

        writer.flush().unwrap()
    }

    // ファイルの打ち上げ
    eprintln!("{} Uploading...", ulid);
    let destination_path = format!("sample/{}.bz2", ulid);
    match upload("sample.txt.bz2",&destination_path,"application/x-bzip2") {
        Ok(()) => eprintln!("{} Done.", ulid),
        Err(x) => eprintln!("{} {}", ulid, x),
    }

    // mysqlに書き込んだサンプル名を登録
    {
        let mut conn = mysql_pool.lock().unwrap().get_conn().unwrap();
        let mut tx = conn.start_transaction(TxOpts::default()).unwrap();

        tx.exec_drop( "INSERT INTO sample (name) VALUES (:name)", params!{"name" => ulid.to_string()} ).unwrap();
        tx.commit().unwrap();
    }
}

impl WriteRecord for GenerationWriter {
    fn write_record(&mut self, record:Record) -> Result<()> {
        self.buffer.push(record);

        if self.buffer.len() >= self.plays_per_write {
            write_samples_flush_buffer( &self.mysql_pool, &self.setting, &self.buffer );
            self.buffer.clear();
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if self.buffer.len() > 0 {
            write_samples_flush_buffer( &self.mysql_pool, &self.setting, &self.buffer );
            self.buffer.clear();
        }

        Ok(())
    }
}
