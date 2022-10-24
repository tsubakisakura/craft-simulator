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
