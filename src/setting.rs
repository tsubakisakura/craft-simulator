use std::sync::Arc;
use std::collections::HashMap;

pub trait AdvanceTable
{
    fn working_reward(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32;
    fn quality_reward(&self, efficiency:u32, high_quality:bool, innovation:bool, grate_strides:bool, inner_quiet:u32) -> u32;
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
}

impl ModifierParameter {
    pub fn new_ishgard_reconstruction_4th() -> ModifierParameter {
        ModifierParameter {
            max_working : 12046,
            max_quality : 81447,
            max_durability : 55,
            max_cp : 569 + 72 + 16,
            advance_table : Arc::new( ApproximationTable {
                work_base : 472, // work_accuracy == 2769
                process_accuracy : 2840 + 70,
                required_process_accuracy : 2540,
            })
        }
    }
}

impl AdvanceTable for ApproximationTable {
    fn working_reward(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32 {
        let cond_rate = if high_progress { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if veneration { 0.5 } else { 0.0 } + if muscle_memory { 1.0 } else { 0.0 };

        return ( self.work_base as f64 * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
    }

    // 効率に対する品質報酬
    // こちらの記事が紹介しているcalculatorの内容を参考にしています。
    // https://jp.finalfantasyxiv.com/lodestone/character/29523439/blog/4641394/
    // 完全一致はしませんが、近似値として使えます。完全一致を求めるならば、データシートを作るほうが良いと思う
    fn quality_reward(&self, efficiency: u32, high_quality: bool, innovation: bool, grate_strides: bool, inner_quiet: u32 ) -> u32 {
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
    fn working_reward(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32 {
        let cond_rate = if high_progress { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if veneration { 0.5 } else { 0.0 } + if muscle_memory { 1.0 } else { 0.0 };

        return ( self.work_base as f64 * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
    }

    fn quality_reward(&self, efficiency:u32, high_quality:bool, innovation:bool, grate_strides:bool, inner_quiet:u32) -> u32 {
        let q3 = *self.quality_base.get( &QualityKey { inner_quiet, efficiency } ).expect("undefined key") as f64;
        let cond_rate = if high_quality { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if grate_strides { 1.0 } else { 0.0 } + if innovation { 0.5 } else { 0.0 };

        return ( q3 * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
    }
}
