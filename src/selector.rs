use std::sync::{Arc,Mutex};

use mysql::*;
use mysql::prelude::*;

use super::network2::*;

#[derive(Debug,Clone)]
pub enum Selector {
    UCB1(f64),
    Optimistic(usize),
    Greedy(usize),
}

#[derive(Clone)]
pub struct UCB1Context {
    mysql_pool : Arc<Mutex<Pool>>,
}

#[derive(Debug)]
pub enum Error {
    MySQLError(mysql::Error),
    InvalidNetworkType(String),
    NotFoundNetworkType(String),
    Empty,
}

impl std::convert::From<mysql::Error> for Error {
    fn from(x: mysql::Error) -> Error {
        Error::MySQLError(x)
    }
}

// UCB1法
// cは探索に使うパラメータで、大きくなればなるほど活用よりも探索を大きく見積もります
fn get_ucb1_model(conn:&mut PooledConn, c:f64) -> std::result::Result<String,Error> {
    // 全状態を取得します
    let res : Vec<(String,f64,f64)> = conn.query(format!("SELECT name, total_reward, total_count FROM evaluation"))?;

    if res.len() == 0 {
        // 何もなければ何もないエラーを返します
        Err(Error::Empty)
    }
    else if let Some((name,_,_)) = res.iter().find(|(_,_,count)| *count == 0.0) {
        // 評価回数0のものがあるならそれを優先します
        Ok(name.clone())
    }
    else {
        // 全て評価済みなのでUCB1最良モデルを計算して返します
        let sum_n : f64 = res.iter().map(|(_,_,count)| count).sum();
        let t = 2.0 * sum_n.ln();
        let (name,_) = res.iter()
            .map(|(name,reward,count)| (Some(name),reward/count + c*(t/count).sqrt()))
            .fold((None,f64::MIN), |(k1,v1), (k2,v2)| if v1 > v2 { (k1,v1) } else { (k2,v2) });

        // 必ず1個はMIN以上なのでfold初期値のNoneが帰ることはないです
        Ok(name.unwrap().clone())
    }
}

// 楽観的初期化法
// nは最良値(==1.0)を取ったとする期待値の回数を指定しておきます
fn get_optimistic_model(conn:&mut PooledConn , n:usize) -> std::result::Result<String,Error> {
    // 1個だけ取得してその結果を返します。ここでvalueは取る必要ない
    let res : Option<(String,f64)> = conn.query_first(format!("SELECT name, (total_reward+{})/(total_count+{}) as value FROM evaluation ORDER BY value DESC LIMIT 1",n,n))?;

    if let Some((name,_)) = res {
        Ok(name)
    }
    else {
        Err(Error::Empty)
    }
}

fn get_greedy_model(conn:&mut PooledConn, threshold:usize) -> std::result::Result<String,Error> {
    // 1個だけ取得してその結果を返します。ここでvalueは取る必要ない
    let res : Option<(String,f64)> = conn.query_first(format!("SELECT name, total_reward/total_count as value FROM evaluation WHERE total_count>={} ORDER BY value DESC LIMIT 1",threshold))?;

    if let Some((name,_)) = res {
        Ok(name)
    }
    else {
        Err(Error::Empty)
    }
}

pub fn get_network_type(conn:&mut PooledConn, name:&str) -> std::result::Result<NetworkType,Error> {
    let res : Option<String> = conn.exec_first("SELECT type FROM network WHERE name=:name", params!{"name"=>name} )?;

    if let Some(x) = res {
        match NetworkType::from_name(&x) {
            Ok(network_type) => Ok(network_type),
            Err(x) => Err(Error::InvalidNetworkType(x)),
        }
    }
    else {
        Err(Error::NotFoundNetworkType(name.to_string()))
    }
}

impl UCB1Context {
    pub fn new( mysql_pool : Arc<Mutex<Pool>> ) -> UCB1Context {
        UCB1Context { mysql_pool : mysql_pool }
    }

    pub fn get_model(&mut self, selector:&Selector) -> std::result::Result<(String,NetworkType),Error> {
        let mut conn = self.mysql_pool.lock().unwrap().get_conn()?;

        let model_name = match *selector {
            Selector::UCB1(x) => get_ucb1_model(&mut conn, x)?,
            Selector::Optimistic(x) => get_optimistic_model(&mut conn, x)?,
            Selector::Greedy(x) => get_greedy_model(&mut conn, x)?,
        };

        let network_type = get_network_type(&mut conn, &model_name)?;
        Ok((model_name,network_type))
    }
}
