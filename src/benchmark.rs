use core::cmp::min;

use super::network2::*;
use super::logic::{State,Setting};

pub struct BenchmarkParameter {
    pub setting:Setting,
    pub batch_size:usize,
    pub plays_per_write:usize,
}

// ベンチマーク
pub fn run_benchmark(param:BenchmarkParameter) {

    let vs = tch::nn::VarStore::new(tch::Device::Cpu);
    let network = TchNetwork::new(&vs.root(), 4);

    let states : Vec<State> = (0..param.batch_size).map( |_| param.setting.initial_state() ).collect();
    let mut remain = param.plays_per_write;

    while remain > 0 {
        let size = min(param.batch_size,remain);
        let _ = network.predict_batch( &states[0..size], &param.setting );
        remain -= size;
    }
}
