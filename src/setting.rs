use std::sync::Arc;
use std::collections::HashMap;

pub trait AdvanceTable
{
    fn working_advance(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32;
    fn quality_advance(&self, efficiency:u32, high_quality:bool, innovation:bool, grate_strides:bool, inner_quiet:u32) -> u32;
}

// イシュガルド第四次復興時に利用していたロジックです。
// 今は使ってないですが残してあります
#[derive(Debug,Clone)]
struct ApproximationTable
{
    // 基本状態の突貫の値を5で割った値を入力します。作業の場合はこの値が分かれば正確に計算できる(と思う、多分)
    work_base : u32,

    // 加工精度をそのまま入力します。
    process_accuracy : u32,

    // 必要加工精度をそのまま入力します。
    required_process_accuracy : u32,
}

// 品質計算用のキー値です。
#[derive(Debug,Clone,Hash,Eq,PartialEq)]
struct QualityKey
{
    // インナークワイエット(0 <= inner_quiet <= 10)
    inner_quiet : u32,

    // 必要な効率キーは以下の通りです。
    // 100 : 加工、倹約加工、精密作業、匠の神業、真価、ヘイスティタッチ
    // 125 : 中級加工
    // 150 : 上級加工、注視加工、集中加工
    // 200 : 下地加工
    // 100 + inner_quiet * 20 (※1 <= inner_quiet <= 10) : ビエルゴの祝福(inner_quietに対応する値だけで良いです。0の時は呼べないので計算不要です。)
    efficiency : u32,
}

// 少し手抜きのテーブルです。実測値を使って小さな誤差で計算できます。
// 高品質・イノベ・グレスト時に僅かですが誤差が発生する場合があります。小数点以下が切り捨てられているためと思われます
#[derive(Debug,Clone)]
struct SimpleTable
{
    // 基本状態の突貫の値を5で割った値を入力します。作業の場合はこの値が分かれば正確に計算できる(と思う、多分)
    work_base : u32,

    // 高品質・イノベ・グレストがない状態での値を入力します。
    quality_base : HashMap<QualityKey,u32>,
}

#[derive(Clone)]
pub struct ModifierParameter
{
    pub max_working : u32,                // 必要工数
    pub max_quality : u32,                // 品質上限
    pub max_durability : u32,             // 初期耐久
    pub max_cp : u32,                     // 初期CP
    pub advance_table : Arc<dyn AdvanceTable + Sync + Send>, // これをArcにしないと多くの関数がGenericになってしまうのでArcにしてます
    pub bonus_time_t : f32,               // 時間ボーナス割合
    pub bonus_threshold_t : f32,          // 閾値ボーナス割合
    pub bonus_threshold : u32,            // 閾値ボーナス最低値
}

impl ModifierParameter {

    // 作業精度2769
    // 加工精度2840
    // maxcp 569
    #[allow(dead_code)]
    pub fn new_ishgard_reconstruction_4th() -> ModifierParameter {
        ModifierParameter {
            max_working : 12046,
            max_quality : 81447,
            max_durability : 55,
            max_cp : 569 + 72 + 16,
            advance_table : Arc::new( ApproximationTable {
                work_base : 472,
                process_accuracy : 2840 + 70,
                required_process_accuracy : 2540,
            }),
            bonus_time_t : 0.15,
            bonus_threshold_t : 0.50,
            bonus_threshold : 81447, // max値の時のみ有効
        }
    }

    // 作業精度3738
    // 加工精度3768
    // maxcp 588
    #[allow(dead_code)]
    pub fn new_fountain_of_usouso() -> ModifierParameter {

        // 横軸100,125,150,200,ビエルゴ
        // 縦軸IQ(0～10)
        let measured_value : [[u32;5];11] = [
            [247,308,370,494,0],
            [271,339,407,543,326],
            [296,370,444,592,414],
            [321,401,481,642,513],
            [345,432,518,691,622],
            [370,463,555,741,741],
            [395,494,592,790,869],
            [419,524,629,839,1007],
            [444,555,666,889,1155],
            [469,586,703,938,1314],
            [494,617,741,988,1482],
        ];

        let mut table = HashMap::new();
        for inner_quiet in 0..=10 {
            for i in 0..5 {
                if !(i == 4 && inner_quiet == 0) {
                    let efficiency = match i {
                        0 => 100,
                        1 => 125,
                        2 => 150,
                        3 => 200,
                        4 => 100 + inner_quiet * 20,
                        _ => panic!("undefined"),
                    };
                    let key = QualityKey { inner_quiet, efficiency };

                    table.entry( key ).or_insert( measured_value[inner_quiet as usize][i] );
                }
            }
        }

        ModifierParameter {
            max_working : 7480,
            max_quality : 13620,
            max_durability : 60,
            max_cp : 588 + 78 + 21,
            advance_table : Arc::new( SimpleTable {
                work_base : 209,
                quality_base : table,
            }),
            bonus_time_t : 0.05,
            bonus_threshold_t : 0.80,
            bonus_threshold : 13500, // ウソウソの泉作成要件
        }
    }
}

impl AdvanceTable for ApproximationTable {
    fn working_advance(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32 {
        let cond_rate = if high_progress { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if veneration { 0.5 } else { 0.0 } + if muscle_memory { 1.0 } else { 0.0 };

        return ( self.work_base as f64 * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
    }

    // 効率に対する品質報酬
    // こちらの記事が紹介しているcalculatorの内容を参考にしています。
    // https://jp.finalfantasyxiv.com/lodestone/character/29523439/blog/4641394/
    // 完全一致はしませんが、近似値として使えます。完全一致を求めるならば、データシートを作るほうが良いと思う
    fn quality_advance(&self, efficiency: u32, high_quality: bool, innovation: bool, grate_strides: bool, inner_quiet: u32 ) -> u32 {
        let iq : f64 = From::from(inner_quiet);
        let process_accuracy : f64 = From::from(self.process_accuracy);
        let required_process_accuracy : f64 = From::from(self.required_process_accuracy);

        let f = process_accuracy + process_accuracy * (iq * 20.0 / 100.0);
        let q1 = f*35.0/100.0 + 35.0;
        let q2 = q1 * (f + 10000.0) / (required_process_accuracy + 10000.0);
        let q3 = q2 * 60.0 / 100.0;
        let cond_rate = if high_quality { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if grate_strides { 1.0 } else { 0.0 } + if innovation { 0.5 } else { 0.0 };

        return ( q3 * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
    }
}

impl AdvanceTable for SimpleTable {
    fn working_advance(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32 {
        let cond_rate = if high_progress { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if veneration { 0.5 } else { 0.0 } + if muscle_memory { 1.0 } else { 0.0 };

        return ( self.work_base as f64 * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
    }

    fn quality_advance(&self, efficiency:u32, high_quality:bool, innovation:bool, grate_strides:bool, inner_quiet:u32) -> u32 {
        let q3 = *self.quality_base.get( &QualityKey { inner_quiet, efficiency } ).expect("undefined key") as f64;
        let cond_rate = if high_quality { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if grate_strides { 1.0 } else { 0.0 } + if innovation { 0.5 } else { 0.0 };

        return ( q3 * cond_rate * buff_rate ) as u32 / 100;
    }
}
