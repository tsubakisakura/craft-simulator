use std::io::{BufReader,Read};

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

const HEADER: [&str; 17] = [
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
    "アート",
    "最終",
    "確信",
    "マニ",
    "状態",
];

fn format_state( s:&State ) -> String {
    format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
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
        s.elements,
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

pub fn run_replay( record_names:Vec<String> ) {
    for record_name in record_names {
        let records = get_records(record_name);

        for record in records {
            write_record( &record );
        }
    }
}
