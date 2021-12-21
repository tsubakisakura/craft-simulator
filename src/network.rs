use super::logic::{State,Setting,Condition};

pub const STATE_NUM : usize = 36;
pub type StateVector = [f32;STATE_NUM];

trait OneHotConvertible {
    fn to_onehot(&self) -> f32;
}

impl OneHotConvertible for u32 {
    fn to_onehot(&self) -> f32 {
        if *self == 0 { 0.0 } else { 1.0 }
    }
}

impl OneHotConvertible for bool {
    fn to_onehot(&self) -> f32 {
        if *self { 1.0 } else { 0.0 }
    }
}

// ターンが絡むものは全て均等に10で割ることにします(各ノードの影響を均等にする意図)
pub fn encode_state( s:&State, setting:&Setting ) -> StateVector {
    [
        s.turn as f32 / 128.0,
        s.time as f32 / 256.0,
        s.completed.to_onehot(), // 要らない気がする
        s.working as f32 / setting.max_working as f32,
        s.quality as f32 / setting.max_quality as f32,
        s.durability as f32 / setting.max_durability as f32,
        s.cp as f32 / setting.max_cp as f32,

        if s.inner_quiet == 0 { 0.0 } else { (s.inner_quiet - 1) as f32 / 10.0 },
        s.inner_quiet.to_onehot(),
        s.careful_observation as f32 / 10.0,
        s.careful_observation.to_onehot(),
        s.waste_not as f32 / 10.0,
        s.waste_not.to_onehot(),
        s.veneration as f32 / 10.0,
        s.veneration.to_onehot(),
        s.great_strides as f32 / 10.0,
        s.great_strides.to_onehot(),
        s.innovation as f32 / 10.0,
        s.innovation.to_onehot(),
        s.elements as f32 / 10.0,
        s.elements.to_onehot(),
        s.final_appraisal as f32 / 10.0,
        s.final_appraisal.to_onehot(),
        s.muscle_memory as f32 / 10.0,
        s.muscle_memory.to_onehot(),
        s.manipulation as f32 / 10.0,
        s.manipulation.to_onehot(),

        s.elements_used.to_onehot(),
        s.combo_touch.to_onehot(),
        s.combo_observe.to_onehot(),

        (s.condition == Condition::Standard).to_onehot(),
        (s.condition == Condition::HighQuality).to_onehot(),
        (s.condition == Condition::HighProgress).to_onehot(),
        (s.condition == Condition::HighEfficiency).to_onehot(),
        (s.condition == Condition::HighSustain).to_onehot(),
        (s.condition == Condition::Solid).to_onehot(),
    ]
}
