use std::error::Error;

use tch::*;
use tch::nn::*;

use super::logic::{State,ACTION_NUM};
use super::setting::ModifierParameter;
use super::encoding::{encode_state_batch,decode_pv_batch};
use super::mcts::*;

pub const STATE_NUM : usize = 36;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum NetworkType {
    FullyConnected(usize,usize),
    Residual(usize,usize),
}

impl NetworkType {

    fn parse_fc(xs:&[&str]) -> Result<Self, String> {
        if xs.len() < 2 {
            return Err("can't parse fc".to_string())
        }

        let depth = match xs[0].parse::<usize>() {
            Ok(x) => Ok(x),
            Err(_) => Err("can't parse depth".to_string()),
        }?;

        let hidden_nodes = match xs[1].parse::<usize>() {
            Ok(x) => Ok(x),
            Err(_) => Err("can't parse hidden_nodes".to_string()),
        }?;

        Ok(NetworkType::FullyConnected(depth, hidden_nodes))
    }

    fn parse_residual(xs:&[&str]) -> Result<Self, String> {
        if xs.len() < 2 {
            return Err("can't parse residual".to_string())
        }

        let depth = match xs[0].parse::<usize>() {
            Ok(x) => Ok(x),
            Err(_) => Err("can't parse depth".to_string()),
        }?;

        let hidden_nodes = match xs[1].parse::<usize>() {
            Ok(x) => Ok(x),
            Err(_) => Err("can't parse hidden_nodes".to_string()),
        }?;

        Ok(NetworkType::Residual(depth, hidden_nodes))
    }

    pub fn from_name(name: &str) -> Result<Self, String> {
        let xs : Vec<&str> = name.split('-').collect();
        if xs.len() < 1 {
            return Err("can't parse NetworkType".to_string())
        }

        match xs[0] {
            "fc" => NetworkType::parse_fc(&xs[1..]),
            "residual" => NetworkType::parse_residual(&xs[1..]),
            _ => Err("unknown network type".to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            NetworkType::FullyConnected(depth,hidden_nodes) => format!("fc-{}-{}", depth, hidden_nodes),
            NetworkType::Residual(depth,hidden_nodes) => format!("residual-{}-{}", depth, hidden_nodes),
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
        NetworkType::FullyConnected(depth, hidden_nodes) => Box::new(FullyConnectedNetwork::new(vs, depth, hidden_nodes)),
        NetworkType::Residual(depth, hidden_nodes) => Box::new(ResidualNetwork::new(vs, depth, hidden_nodes)),
    }
}

pub trait DualNetwork {

    fn forward_t(&self, input: &Tensor, train:bool) -> (Tensor,Tensor);

    fn predict_batch(&self, states:&[State], mod_param:&ModifierParameter) -> Result<Vec<(ActionVector,f32)>, Box<dyn Error>> {
        let state_vec_t = encode_state_batch( states, mod_param );
        let pv_t = self.forward_t(&state_vec_t, false);
        Ok(decode_pv_batch(pv_t))
    }
}

pub struct FullyConnectedNetwork {
    main_net:SequentialT,
    policy_net:SequentialT,
    value_net:SequentialT,
}

fn create_main_network(vs: &nn::Path, depth: usize, hidden_nodes: usize) -> SequentialT {
    let hidden_nodes = hidden_nodes as i64;

    if depth == 0 {
        panic!("depth must be greater than 0");
    }

    let mut net = nn::seq_t()
        .add(nn::linear( vs / "layer0", STATE_NUM as i64, hidden_nodes, Default::default()))
        .add_fn(|xs| xs.relu());

    for i in 1..depth {
        net = net
            .add(nn::linear( vs / format!("layer{}",i), hidden_nodes, hidden_nodes, Default::default()))
            .add_fn(|xs| xs.relu());
    }

    net.add_fn_t(|xs, train| xs.dropout(0.1, train))
}

fn create_policy_network(vs: &nn::Path, hidden_nodes: usize) -> SequentialT {
    nn::seq_t()
        .add(nn::linear( vs / "policy", hidden_nodes as i64, ACTION_NUM as i64, Default::default()))
        .add_fn(|xs| xs.softmax(1,Kind::Float))
}

fn create_value_network(vs: &nn::Path, hidden_nodes: usize) -> SequentialT {
    nn::seq_t()
        .add(nn::linear( vs / "value", hidden_nodes as i64, 1, Default::default()))
        .add_fn(|xs| xs.sigmoid())
}

impl FullyConnectedNetwork {
    pub fn new(vs: &nn::Path, depth: usize, hidden_nodes: usize) -> FullyConnectedNetwork {
        FullyConnectedNetwork {
            main_net: create_main_network(vs, depth, hidden_nodes),
            policy_net: create_policy_network(vs, hidden_nodes),
            value_net: create_value_network(vs, hidden_nodes),
        }
    }
}

impl DualNetwork for FullyConnectedNetwork {
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
    pub fn new(vs: &nn::Path, hidden_nodes: usize) -> ResidualUnit {
        let hidden_nodes = hidden_nodes as i64;
        ResidualUnit { main_net: nn::seq_t()
            .add(nn::batch_norm1d( vs / "bn1", hidden_nodes, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear( vs / "layer1", hidden_nodes, hidden_nodes, Default::default()))
            .add(nn::batch_norm1d( vs / "bn2", hidden_nodes, Default::default()))
            .add_fn(|xs| xs.relu())
            .add_fn_t(|xs, train| xs.dropout(0.3, train))
            .add(nn::linear( vs / "layer2", hidden_nodes, hidden_nodes, Default::default()))
        }
    }

    fn forward_t(&self, input: &Tensor, train:bool) -> Tensor {
        self.main_net.forward_t(input, train) + input
    }
}

impl ResidualNetwork {
    pub fn new(vs: &nn::Path, depth: usize, hidden_nodes: usize) -> ResidualNetwork {
        let input_net = nn::linear( vs / "input", STATE_NUM as i64, hidden_nodes as i64, Default::default());
        let residual_units = (0..depth).into_iter().map(|i| ResidualUnit::new(&(vs/format!("residual_units_{}",i)),hidden_nodes)).collect();
        let policy_net = create_policy_network(vs, hidden_nodes);
        let value_net = create_value_network(vs, hidden_nodes);

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
