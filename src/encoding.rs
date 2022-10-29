use tch::*;

use super::logic::{State,Condition,ACTION_NUM};
use super::setting::ModifierParameter;
use super::mcts::*;

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
pub fn encode_state( s:&State, mod_param:&ModifierParameter ) -> StateVector {
    [
        s.turn as f32 / 128.0,
        s.time as f32 / 256.0,
        s.completed.to_onehot(), // 要らない気がする
        s.working as f32 / mod_param.max_working as f32,
        s.quality as f32 / mod_param.max_quality as f32,
        s.durability as f32 / mod_param.max_durability as f32,
        s.cp as f32 / mod_param.max_cp as f32,

        s.inner_quiet as f32 / 10.0,
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
        s.final_appraisal as f32 / 10.0,
        s.final_appraisal.to_onehot(),
        s.muscle_memory as f32 / 10.0,
        s.muscle_memory.to_onehot(),
        s.manipulation as f32 / 10.0,
        s.manipulation.to_onehot(),

        s.heart_and_soul.to_onehot(),
        s.heart_and_soul_used.to_onehot(),
        s.combo_basic_touch.to_onehot(),
        s.combo_standard_touch.to_onehot(),
        s.combo_observe.to_onehot(),

        (s.condition == Condition::Standard).to_onehot(),
        (s.condition == Condition::HighQuality).to_onehot(),
        (s.condition == Condition::HighProgress).to_onehot(),
        (s.condition == Condition::HighEfficiency).to_onehot(),
        (s.condition == Condition::HighSustain).to_onehot(),
        (s.condition == Condition::Solid).to_onehot(),
    ]
}

// 配列からテンソル作成
// あまり効率はよくない
pub fn encode_state_batch( states:&[State], mod_param:&ModifierParameter ) -> Tensor {

    let mut state_vec = vec!{};
    state_vec.resize( states.len() * STATE_NUM, 0.0 );

    for i in 0..states.len() {
        state_vec[i*STATE_NUM..(i+1)*STATE_NUM].copy_from_slice( &encode_state(&states[i],mod_param) );
    }

    Tensor::of_slice(&state_vec).reshape(&[states.len() as i64, STATE_NUM as i64])
}

fn convert_to_policy_vector( t:&Tensor, offset:i64 ) -> ActionVector {
    let mut res = [0.0;ACTION_NUM];
    t.slice(0,Some(offset), Some(offset+1), 1).copy_data(&mut res, ACTION_NUM);
    res
}

pub fn decode_pv_batch( (policy_res_t,value_res_t):(Tensor,Tensor) ) -> Vec<(ActionVector,f32)> {
    let policy_iter = (0..policy_res_t.size2().unwrap().0).into_iter().map(|i| convert_to_policy_vector(&policy_res_t,i));
    let value_iter = (0..value_res_t.size2().unwrap().0).into_iter().map(|i| value_res_t.double_value(&[i as i64,0]) as f32);

    policy_iter.zip(value_iter).collect()
}
