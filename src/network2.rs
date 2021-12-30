use std::error::Error;

use tch::*;
use tch::nn::*;

use super::logic::*;
use super::network::{encode_state_batch,decode_pv_batch};
use super::mcts::*;

pub const STATE_NUM : usize = 36;
const HIDDEN_NODES: i64 = 128;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum NetworkType {
    FullyConnected,
    Residual,
}

impl NetworkType {
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name {
            "fully-connected" => Ok(NetworkType::FullyConnected),
            "residual" => Ok(NetworkType::Residual),
            _ => Err("unknown network type".to_string()),
        }
    }
}

impl argh::FromArgValue for NetworkType {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        NetworkType::from_name(value)
    }
}

pub fn create_network(vs: &nn::Path, network_type: NetworkType) -> Box<dyn DualNetwork> {
    match network_type {
        NetworkType::FullyConnected => Box::new(TchNetwork::new(vs)),
        NetworkType::Residual => Box::new(ResidualNetwork::new(vs)),
    }
}

pub trait DualNetwork {

    fn forward_t(&self, input: &Tensor, train:bool) -> (Tensor,Tensor);

    fn predict_batch(&self, states:&[State], setting:&Setting) -> Result<Vec<(ActionVector,f32)>, Box<dyn Error>> {
        let state_vec_t = encode_state_batch( states, setting );
        let pv_t = self.forward_t(&state_vec_t, false);
        Ok(decode_pv_batch(pv_t))
    }
}

pub struct TchNetwork {
    main_net:SequentialT,
    policy_net:SequentialT,
    value_net:SequentialT,
}

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

impl TchNetwork {
    pub fn new(vs: &nn::Path) -> TchNetwork {
        TchNetwork {
            main_net: create_main_network(vs),
            policy_net: create_policy_network(vs),
            value_net: create_value_network(vs),
        }
    }
}

impl DualNetwork for TchNetwork {
    fn forward_t(&self, input: &Tensor, train:bool) -> (Tensor,Tensor) {
        let main_output = self.main_net.forward_t(input, train);
        let policy_output = self.policy_net.forward_t(&main_output, train);
        let value_output = self.value_net.forward_t(&main_output, train);

        (policy_output, value_output)
    }
}

struct ResidualUnit {
    main_net : SequentialT,
}

pub struct ResidualNetwork {
    input_net: nn::Linear,
    residual_units: Vec<ResidualUnit>,
    policy_net: SequentialT,
    value_net: SequentialT,
}

impl ResidualUnit {
    pub fn new(vs: &nn::Path) -> ResidualUnit {
        ResidualUnit { main_net: nn::seq_t()
            .add(nn::batch_norm1d( vs / "bn1", HIDDEN_NODES, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear( vs / "layer1", HIDDEN_NODES, HIDDEN_NODES, Default::default()))
            .add(nn::batch_norm1d( vs / "bn2", HIDDEN_NODES, Default::default()))
            .add_fn(|xs| xs.relu())
            .add_fn_t(|xs, train| xs.dropout(0.3, train))
            .add(nn::linear( vs / "layer2", HIDDEN_NODES, HIDDEN_NODES, Default::default()))
        }
    }

    fn forward_t(&self, input: &Tensor, train:bool) -> Tensor {
        self.main_net.forward_t(input, train) + input
    }
}

impl ResidualNetwork {
    pub fn new(vs: &nn::Path) -> ResidualNetwork {
        let input_net = nn::linear( vs / "input", STATE_NUM as i64, HIDDEN_NODES, Default::default());
        let residual_units = (0..4).into_iter().map(|i| ResidualUnit::new(&(vs/format!("residual_units_{}",i)))).collect();
        let policy_net = create_policy_network(vs);
        let value_net = create_value_network(vs);

        ResidualNetwork { input_net, residual_units, policy_net, value_net }
    }
}

impl DualNetwork for ResidualNetwork {
    fn forward_t(&self, input: &Tensor, train:bool) -> (Tensor,Tensor) {
        let input_output = self.input_net.forward_t(input, train);
        let main_output = self.residual_units.iter().fold(input_output, |x,unit| unit.forward_t(&x, train));
        let policy_output = self.policy_net.forward_t(&main_output, train);
        let value_output = self.value_net.forward_t(&main_output, train);

        (policy_output, value_output)
    }
}
