use core::cmp::min;

use super::network::Network;
use super::logic::{State,Setting};

pub struct BenchmarkParameter {
    pub setting:Setting,
    pub batch_size:usize,
    pub plays_per_write:usize,
}

// ベンチマーク
pub fn run_benchmark(param:BenchmarkParameter) {

    let graph = super::network::load_graph_from_file("model.pb").unwrap();
    let network = Network::load_graph(&graph).unwrap();

    if param.batch_size == 0 {
        // バッチサイズが0の場合は単独で推論します。こちらの場合はVecを使わないから単独推論としてはとても高速
        let state = param.setting.initial_state();

        for _ in 0..param.plays_per_write {
            let _ = network.predict( &state, &param.setting );
        }
    }
    else {
        // バッチサイズが0より大きい場合、バッチ処理で推論します。Vecを使う分だけ少し遅いです。
        let states : Vec<State> = (0..param.batch_size).map( |_| param.setting.initial_state() ).collect();
        let mut remain = param.plays_per_write;

        while remain > 0 {
            let size = min(param.batch_size,remain);
            let _ = network.predict_batch( &states[0..size], &param.setting );
            remain -= size;
        }
    }
}
