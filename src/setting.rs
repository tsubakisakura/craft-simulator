#[derive(Debug,Clone)]
pub struct Setting
{
    pub max_working : u32,                // 必要工数
    pub max_quality : u32,                // 品質上限
    pub max_durability : u32,             // 初期耐久
    pub work_accuracy : u32,              // 作業精度
    pub process_accuracy : u32,           // 加工精度
    pub required_process_accuracy : u32,  // 必要加工精度
    pub max_cp : u32,                     // 初期CP
}

pub trait AdvanceTable
{
    fn working_reward(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32;
    fn quality_reward(&self, efficiency:u32, high_quality:bool, innovation:bool, grate_strides:bool, inner_quiet:u32) -> u32;
}

#[derive(Debug,Clone)]
pub struct ModifierParameter
{
    pub max_working : u32,                // 必要工数
    pub max_quality : u32,                // 品質上限
    pub max_durability : u32,             // 初期耐久
    pub work_accuracy : u32,              // 作業精度
    pub process_accuracy : u32,           // 加工精度
    pub required_process_accuracy : u32,  // 必要加工精度
    pub max_cp : u32,                     // 初期CP
}

pub fn initial_setting() -> Setting {
    Setting {
        max_working:12046,
        max_quality:81447,
        max_durability:55,
        work_accuracy:2769,
        //process_accuracy:2840,
        process_accuracy:2840 + 70,
        required_process_accuracy:2540,
        //max_cp:569,
        max_cp:569 + 72 + 16,
    }
}

impl ModifierParameter {
    pub fn new(setting:&Setting) -> ModifierParameter {
        ModifierParameter {
            max_working : setting.max_working,
            max_quality : setting.max_quality,
            max_durability : setting.max_durability,
            work_accuracy : setting.work_accuracy,
            process_accuracy : setting.process_accuracy,
            required_process_accuracy : setting.required_process_accuracy,
            max_cp : setting.max_cp,
        }
    }
}

impl AdvanceTable for ModifierParameter {
    fn working_reward(&self, efficiency:u32, high_progress:bool, veneration:bool, muscle_memory:bool) -> u32 {
        // 情報が無いのでそのまま決め打ちの数値の対応です。それ以外に対応することになったらやる
        if self.work_accuracy != 2769 {
            return 99999;
        }

        let q = 472.0;
        let cond_rate = if high_progress { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if veneration { 0.5 } else { 0.0 } + if muscle_memory { 1.0 } else { 0.0 };

        return ( q * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
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
