use std::error::Error;

use tch::*;
use tch::nn::*;

use super::logic::*;
use super::network::encode_state;
use super::mcts::*;

pub struct TchNetwork {
    main_net:SequentialT,
    policy_net:SequentialT,
    value_net:SequentialT,
}

pub const STATE_NUM : usize = 36;
const HIDDEN_NODES: i64 = 128;

fn create_main_network(vs: &nn::Path) -> SequentialT {
    nn::seq_t()
        .add(nn::linear( vs / "layer1", STATE_NUM as i64, HIDDEN_NODES, Default::default()))
        .add_fn(|xs| xs.relu())
        .add(nn::linear( vs / "layer2", HIDDEN_NODES, HIDDEN_NODES, Default::default()))
        .add_fn(|xs| xs.relu())
        .add(nn::linear( vs / "layer3", HIDDEN_NODES, HIDDEN_NODES, Default::default()))
        .add_fn(|xs| xs.relu())
        .add(nn::linear( vs / "layer4", HIDDEN_NODES, HIDDEN_NODES, Default::default()))
        .add_fn(|xs| xs.relu())
        .add_fn_t(|xs, train| xs.dropout(0.1, train))
}

fn create_policy_network(vs: &nn::Path) -> SequentialT {
    nn::seq_t()
        .add(nn::linear( vs / "policy", HIDDEN_NODES, ACTION_NUM as i64, Default::default()))
        .add_fn(|xs| xs.softmax(1,Kind::Float))
}

fn create_value_network(vs: &nn::Path) -> SequentialT {
    nn::seq_t()
        .add(nn::linear( vs / "value", HIDDEN_NODES, 1, Default::default()))
        .add_fn(|xs| xs.sigmoid())
}

// 配列からテンソル作成
// あまり効率はよくない
fn encode_state_batch( states:&[State], setting:&Setting ) -> Tensor {

    let mut state_vec = vec!{};
    state_vec.resize( states.len() * STATE_NUM, 0.0 );

    for i in 0..states.len() {
        state_vec[i*STATE_NUM..(i+1)*STATE_NUM].copy_from_slice( &encode_state(&states[i],setting) );
    }

    Tensor::of_slice(&state_vec).reshape(&[states.len() as i64, STATE_NUM as i64])
}

fn convert_to_policy_vector( t:&Tensor, offset:i64 ) -> ActionVector {
    let mut res = [0.0;ACTION_NUM];
    t.slice(0,Some(offset), Some(offset+1), 1).copy_data(&mut res, ACTION_NUM);
    res
}

fn decode_pv_batch( (policy_res_t,value_res_t):(Tensor,Tensor) ) -> Vec<(ActionVector,f32)> {
    let policy_iter = (0..policy_res_t.size2().unwrap().0).into_iter().map(|i| convert_to_policy_vector(&policy_res_t,i));
    let value_iter = (0..value_res_t.size2().unwrap().0).into_iter().map(|i| value_res_t.double_value(&[i as i64,0]) as f32);

    policy_iter.zip(value_iter).collect()
}

impl TchNetwork {
    pub fn new(vs: &nn::Path) -> TchNetwork {
        TchNetwork {
            main_net: create_main_network(vs),
            policy_net: create_policy_network(vs),
            value_net: create_value_network(vs),
        }
    }

    pub fn forward_t(&self, input: &Tensor, train:bool) -> (Tensor,Tensor) {
        let main_output = self.main_net.forward_t(input, train);
        let policy_output = self.policy_net.forward_t(&main_output, train);
        let value_output = self.value_net.forward_t(&main_output, train);

        (policy_output, value_output)
    }

    pub fn predict_batch(&self, states:&[State], setting:&Setting) -> Result<Vec<(ActionVector,f32)>, Box<dyn Error>> {
        let state_vec_t = encode_state_batch( states, setting );
        let pv_t = self.forward_t(&state_vec_t, false);
        Ok(decode_pv_batch(pv_t))
    }
}
