use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context,Poll};
use std::cell::{Cell,RefCell};
use std::rc::Rc;

use super::mcts::ActionVector;
use super::logic::{State,Setting};
use super::network2::*;

// 個々のNNが予測した結果を保存するための場所
// PendingおよびReadyがそのまま入っています。実質Optionと一緒。
// そのままFutureの戻り値として使えます。
#[derive(Clone)]
pub struct PredictResult {
    res : Rc<Cell<Poll<(ActionVector,f32)>>>
}

impl PredictResult {
    pub fn new() -> PredictResult {
        PredictResult { res : Rc::new(Cell::new(Poll::Pending)) }
    }
}

impl Future for PredictResult {
    type Output = (ActionVector,f32);

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<(ActionVector,f32)> {
        ctx.waker().wake_by_ref();
        self.res.get()
    }
}

// 予測システム
pub struct Predictor {
    networks : HashMap<String,(tch::nn::VarStore,Box<dyn DualNetwork>)>,
    tasks : Rc<RefCell<HashMap<String,Vec<(State,PredictResult)>>>>,
}

#[derive(Clone)]
pub struct PredictQueue {
    tasks : Rc<RefCell<HashMap<String,Vec<(State,PredictResult)>>>>,
}

impl Predictor {
    pub fn new() -> Predictor {
        Predictor { networks : HashMap::new(), tasks:Rc::new(RefCell::new(HashMap::new())) }
    }

    pub fn load_network(&mut self, name:String, (network_type,source_vs):&(NetworkType,tch::nn::VarStore) ) {
        if !self.networks.contains_key(&name) {
            let mut vs = tch::nn::VarStore::new(tch::Device::Cpu);
            let net = create_network(&vs.root(), *network_type);
            vs.copy(source_vs).unwrap(); // ファイルから直接読んでも良いです。どうせ全体から見るとどちらも大差ない
            self.networks.insert(name, (vs,net) );
        }
    }

    pub fn predict_batch(&mut self, setting:&Setting) {
        let mut tasks = self.tasks.borrow_mut();

        for (name,task_vec) in tasks.iter() {
            // ここでnameに対応するnetworkは絶対に見つかる想定です。
            // ここで見つからない場合はロジックがおかしいので処理を見直します
            let network = self.networks.get(name).expect("not found network");
            let (source,results) : (Vec<State>, Vec<PredictResult>) = task_vec.iter().cloned().unzip();
            let dest = network.1.predict_batch( &source, setting ).unwrap();

            for (result,d) in results.iter().zip( dest.iter() ) {
                result.res.set(Poll::Ready(*d))
            }
        }

        tasks.clear();
    }

    pub fn get_queue(&self) -> PredictQueue {
        PredictQueue { tasks : self.tasks.clone() }
    }
}

impl PredictQueue {
    pub async fn async_predict( &self, name:String, x:State ) -> (ActionVector,f32) {
        let pr = PredictResult::new();
        self.tasks.borrow_mut().entry(name).or_insert(Vec::new()).push( (x,pr.clone()) );
        pr.await
    }
}
