use std::sync::{Arc,Mutex};

use mysql::*;
use mysql::prelude::*;

#[derive(Debug,Clone)]
pub enum Selector {
    UCB1(f64),
    Optimistic(usize),
}

#[derive(Clone)]
pub struct UCB1Context {
    mysql_pool : Arc<Mutex<Pool>>,
}

impl UCB1Context {
    pub fn new( mysql_pool : Arc<Mutex<Pool>> ) -> UCB1Context {
        UCB1Context { mysql_pool : mysql_pool }
    }

    // UCB1法
    // cは探索に使うパラメータで、大きくなればなるほど活用よりも探索を大きく見積もります
    pub fn get_ucb1_model(&mut self, c:f64) -> mysql::Result<Option<String>> {
        let mut conn = self.mysql_pool.lock().unwrap().get_conn()?;

        // 全状態を取得します
        let res : Vec<(String,f64,f64)> = conn.query(format!("SELECT name, total_reward, total_count FROM evaluation"))?;

        if res.len() == 0 {
            // 何もなければ何もしません
            Ok(None)
        }
        else if let Some((name,_,_)) = res.iter().find(|(_,_,count)| *count == 0.0) {
            // 評価回数0のものがあるならそれを優先します
            Ok(Some(name.clone()))
        }
        else {
            // 全て評価済みなのでUCB1最良モデルを計算して返します
            let sum_n : f64 = res.iter().map(|(_,_,count)| count).sum();
            let t = 2.0 * sum_n.ln();
            let (name,_) = res.iter()
                .map(|(name,reward,count)| (Some(name),reward/count + c*(t/count).sqrt()))
                .fold((None,f64::MIN), |(k1,v1), (k2,v2)| if v1 > v2 { (k1,v1) } else { (k2,v2) });

            // 必ず1個はMIN以上なのでfold初期値のNoneが帰ることはないです
            Ok(Some(name.unwrap().clone()))
        }
    }

    // 楽観的初期化法
    // nは最良値(==1.0)を取ったとする期待値の回数を指定しておきます
    pub fn get_optimistic_model(&mut self, n:usize) -> mysql::Result<Option<String>> {
        let mut conn = self.mysql_pool.lock().unwrap().get_conn()?;

        // 1個だけ取得してその結果を返します。ここでvalueは取る必要ない
        let res : Option<(String,f64)> = conn.query_first(format!("SELECT name, (total_reward+{})/(total_count+{}) as value FROM evaluation ORDER BY value DESC LIMIT 1",n,n))?;

        if let Some((name,_)) = res {
            Ok(Some(name))
        }
        else {
            Ok(None)
        }
    }
}
