use std::io::{BufReader,Read};
use std::collections::HashMap;
use num::FromPrimitive;

use bzip2::read::BzDecoder;

use super::logic::*;
use super::selfplay::*;
use super::gcs::*;

fn get_records( record_name: String ) -> Vec<Record> {
    eprintln!("{} Downloading...", record_name);

    // レコード取得
    let path = format!("record/{}.bz2", record_name);
    std::fs::create_dir_all("record").unwrap();
    download(&path,&path).unwrap();

    eprintln!("{} Done.", record_name);

    // デコード
    let file = std::fs::File::open(&path).unwrap();
    let mut reader = BufReader::new(BzDecoder::new(file));
    let mut serialized = Vec::new();
    reader.read_to_end(&mut serialized).unwrap();

    // デシリアライズ
    bincode::deserialize(&serialized).unwrap()
}

const HEADER: [&str; 16] = [
    "TURN",
    "時間",
    "作業",
    "品質",
    "耐久",
    "CP",
    "IQ",
    "設計",
    "倹約",
    "ヴェネ",
    "グレ",
    "イノベ",
    "最終",
    "確信",
    "マニ",
    "状態",
];

fn format_state( s:&State ) -> String {
    format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        s.turn,
        s.time,
        s.working,
        s.quality,
        s.durability,
        s.cp,
        s.inner_quiet,
        s.careful_observation,
        s.waste_not,
        s.veneration,
        s.great_strides,
        s.innovation,
        s.final_appraisal,
        s.muscle_memory,
        s.manipulation,
        s.condition.translate_ja(),
    )
}

fn write_record( record: &Record ) {
    println!("{}", HEADER.join("\t").to_string());

    for sample in &record.samples {
        println!("{}\t{}", format_state(&sample.state), sample.action.translate_ja());
    }
}

fn count_skill_histogram( counter: &mut HashMap<(Action,Condition),u32>, record: &Record ) {
    for sample in &record.samples {
        let key = (sample.action, sample.state.condition);
        *counter.entry(key).or_insert(0) += 1;
    }
}

fn write_skill_histogram( counter: &HashMap<(Action,Condition),u32> ) {

    println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        "アクション",
        Condition::Standard.translate_ja(),
        Condition::HighQuality.translate_ja(),
        Condition::HighProgress.translate_ja(),
        Condition::HighEfficiency.translate_ja(),
        Condition::HighSustain.translate_ja(),
        Condition::Solid.translate_ja(),
        Condition::Stable.translate_ja());

    for a in 0..ACTION_NUM {
        let action = Action::from_u64(a as u64).unwrap();

        println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            action.translate_ja(),
            counter.get(&(action,Condition::Standard)).unwrap_or(&0),
            counter.get(&(action,Condition::HighQuality)).unwrap_or(&0),
            counter.get(&(action,Condition::HighProgress)).unwrap_or(&0),
            counter.get(&(action,Condition::HighEfficiency)).unwrap_or(&0),
            counter.get(&(action,Condition::HighSustain)).unwrap_or(&0),
            counter.get(&(action,Condition::Solid)).unwrap_or(&0),
            counter.get(&(action,Condition::Stable)).unwrap_or(&0));
    }
}

pub fn run_replay( record_names:Vec<String> ) {

    let mut counter : HashMap<(Action,Condition),u32> = HashMap::new();

    for record_name in record_names {
        let records = get_records(record_name);

        for record in records {
            write_record( &record );
            count_skill_histogram( &mut counter, &record );
        }
    }

    write_skill_histogram( &counter );
}
