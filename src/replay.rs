use std::io::{BufReader,Read};

use bzip2::read::BzDecoder;

use super::selfplay::*;
use super::gcs::*;

fn get_records( record_name: String ) -> Vec<Record> {
    println!("{} Downloading...", record_name);

    // レコード取得
    let path = format!("record/{}.bz2", record_name);
    std::fs::create_dir_all("record").unwrap();
    download(&path,&path).unwrap();

    println!("{} Done.", record_name);

    // デコード
    let file = std::fs::File::open(&path).unwrap();
    let mut reader = BufReader::new(BzDecoder::new(file));
    let mut serialized = Vec::new();
    reader.read_to_end(&mut serialized).unwrap();

    // デシリアライズ
    bincode::deserialize(&serialized).unwrap()
}

pub fn run_replay( record_name:String ) {
    let records = get_records(record_name);
    println!("{:?}", records );
}
