use std::io::{BufReader,BufRead};
use std::sync::{Arc,Mutex};
use std::time::{Instant};

use bzip2::read::BzDecoder;
use mysql::*;
use mysql::prelude::*;
use tch::*;
use tch::nn::*;
use ulid::*;

use super::gcs::*;
use super::network2::*;
use super::logic::*;

pub struct LearnerParameter {
    pub epochs_per_write : usize,
    pub replay_buffer_size : usize,
    pub mysql_user : String,
}

struct ReplayBuffer {
    states:Tensor,
    policies:Tensor,
    values:Tensor,
    max_length:i64,
    last_sample_blob:Option<String>, // 不格好だけれど一旦ここで定義します。いつか分離したい
}

impl ReplayBuffer {
    fn new( max_length: usize ) -> ReplayBuffer {
        ReplayBuffer {
            states: Tensor::zeros(&[0_i64, STATE_NUM as i64], (Kind::Float, Device::Cpu)),
            policies: Tensor::zeros(&[0_i64, ACTION_NUM as i64], (Kind::Float, Device::Cpu)),
            values: Tensor::zeros(&[0_i64, 1], (Kind::Float, Device::Cpu)),
            max_length: max_length as i64,
            last_sample_blob: None,
        }
    }

    fn len(&self) -> i64 {
        self.states.size2().unwrap().0
    }

    fn is_empty(&self) -> bool {
        self.states.size2().unwrap().0 == 0
    }

    fn append(&mut self, (states,policies,values):(Tensor,Tensor,Tensor) ) {
        self.states = Tensor::cat(&[self.states.shallow_clone(),states],0);
        self.policies = Tensor::cat(&[self.policies.shallow_clone(),policies],0);
        self.values = Tensor::cat(&[self.values.shallow_clone(),values],0);

        // 第１列目が想定より大きければ削ります
        let length = self.len();
        if length > self.max_length {
            let start = length - self.max_length;
            let end = length;
            self.states = self.states.slice(0, start, end, 1);
            self.policies = self.policies.slice(0, start, end, 1);
            self.values = self.values.slice(0, start, end, 1);
        }
    }
}

// ファイルからサンプルを読み込みます。
// 各テンソルの大きさは行数をNとして(N,STATE_NUM),(N,ACTION_NUM),(N,1)となります。
// 最初に全要素をfloatで読み取り、それをreshapeして、最後に分割します。
pub fn load_samples<R:BufRead>( reader:R ) -> (Tensor,Tensor,Tensor) {
    let mut data : Vec<f32> = Vec::new();

    eprintln!("read file...");

    // まずVecとして読み込みます
    for line in reader.lines() {
        line.unwrap().split_whitespace().for_each(|x| data.push(x.parse().ok().unwrap()));
    }

    eprintln!("create tensors...");

    // Tensorに変換
    let line_size = STATE_NUM+ACTION_NUM+1;
    let line_num = data.len() / line_size;

    let mut samples = Tensor::of_slice(&data);
    let _ = samples.resize_(&[line_num as i64,line_size as i64]);
    eprintln!("load samples: {:?}", samples.size() );

    let tmp = samples.split_with_sizes(&[STATE_NUM as i64,ACTION_NUM as i64, 1], 1);
    (tmp[0].shallow_clone(),tmp[1].shallow_clone(),tmp[2].shallow_clone())
}

fn download_samples( blob_name:&String ) -> (Tensor,Tensor,Tensor) {
    let path = format!("sample/{}.bz2", blob_name);
    eprintln!("download: {}", path);
    download( &path, "sample.txt.bz2" ).unwrap();
    let file = std::fs::File::open("sample.txt.bz2").unwrap();
    let reader = BufReader::new(BzDecoder::new(file));
    load_samples(reader)
}

fn loss_policy(p_pred:&Tensor, p_true:&Tensor) -> Tensor {
    (-p_true * (p_pred+0.0001).log()).sum_dim_intlist( &[1], false, Kind::Double ).mean(Kind::Double)
}

fn loss_value(v_pred:&Tensor, v_true:&Tensor) -> Tensor {
    (v_pred - v_true).square().mean(Kind::Double)
}

fn loss_alphazero(p_pred:&Tensor, p_true:&Tensor, v_pred:&Tensor, v_true:&Tensor) -> (Tensor,Tensor,Tensor) {
    let p_loss = loss_policy(p_pred, p_true);
    let v_loss = loss_value(v_pred, v_true);

    (p_loss.shallow_clone() + v_loss.shallow_clone(), p_loss, v_loss)
}

fn get_new_samples( mysql_pool:&Arc<Mutex<Pool>>, last_sample_blob:&Option<String> ) -> mysql::Result<Vec<String>> {
    let mut conn = mysql_pool.lock().unwrap().get_conn()?;

    match last_sample_blob {
        Some(x) => conn.exec("SELECT name FROM sample WHERE name>:name", params!{"name" => x}),
        None => conn.query("SELECT name FROM sample"),
    }
}

fn is_exist_model( mysql_pool:&Arc<Mutex<Pool>> ) -> mysql::Result<bool> {
    let mut conn = mysql_pool.lock().unwrap().get_conn()?;
    let ret : usize = conn.query_first(format!("SELECT COUNT(*) FROM evaluation"))?.unwrap();

    Ok( ret > 0 )
}

fn add_samples_from_blobs( replay_buffer:&mut ReplayBuffer, blobs:&[String], read_before:i64 ) {
    if blobs.len() == 0 {
        return
    }

    let samples = download_samples( blobs.last().unwrap() );
    let read_sum = read_before + samples.0.size2().unwrap().0;

    if read_sum < replay_buffer.max_length {
        add_samples_from_blobs( replay_buffer, &blobs[0..blobs.len()-1], read_sum )
    }

    replay_buffer.append( samples );
    replay_buffer.last_sample_blob = Some(blobs.last().unwrap().clone());
}

fn train( optimizer:&mut Optimizer, net:&dyn DualNetwork, replay_buffer:&ReplayBuffer, epoch_num:usize ) {
    eprintln!("train for replay buffer size: {}", replay_buffer.len());

    let mut start = Instant::now();

    for epoch in 0..epoch_num {
        let (p,v) = net.forward_t(&replay_buffer.states,true);
        let (loss,p_loss,v_loss) = loss_alphazero(&p, &replay_buffer.policies, &v, &replay_buffer.values);
        optimizer.backward_step(&loss);

        let now = Instant::now();
        let elapsed_time = now - start;
        eprintln!( "epoch: {:4}, elapsed_time[msec]: {}, loss: {:8.5}, p_loss: {:8.5}, v_loss: {:8.5}", epoch, elapsed_time.as_millis(), f64::from(&loss), f64::from(&p_loss), f64::from(&v_loss) );
        start = now;
    }
}

fn export_weights( mysql_pool:&Arc<Mutex<Pool>>, vs:&VarStore ) -> mysql::Result<()> {
    let ulid = Ulid::new();
    eprintln!("uploading weights... {}", ulid);
    vs.save("weights").unwrap();
    upload("weights", &format!("weights/{}", ulid), "application/x-weights").unwrap();

    let mut conn = mysql_pool.lock().unwrap().get_conn()?;
    conn.exec_drop("INSERT evaluation (name, total_reward, total_count) VALUES (:name,0,0)", params!{"name" => ulid.to_string()})
}

fn run_epoch_loop( mysql_pool:&Arc<Mutex<Pool>>, replay_buffer:&mut ReplayBuffer, optimizer:&mut Optimizer, vs:&VarStore, net:&dyn DualNetwork, epoch:usize ) {
    eprintln!("enumerate sample files from mysql...");
    let sample_blobs = get_new_samples( mysql_pool, &replay_buffer.last_sample_blob ).unwrap();

    eprintln!("download samples...");
    add_samples_from_blobs( replay_buffer, &sample_blobs, 0 );

    if !replay_buffer.is_empty() {
        // バッファに何かあるなら学習して出力します。
        train( optimizer, net, replay_buffer, epoch );
        export_weights( &mysql_pool, vs ).unwrap();
    }
    else if !is_exist_model(mysql_pool).unwrap() {
        // バッファに何もなく、モデルもないなら、今のモデルを初期状態として出力します。
        export_weights( &mysql_pool, vs ).unwrap();
    }
    else {
        // バッファに何もないけど、モデルはある状態です。
        // evaluatorとgeneratorは最良モデルを利用して計算しているはずなので、新しいサンプルの到着を待ちます。
        eprintln!("Wait for new samples...");
        std::thread::sleep( std::time::Duration::from_secs(3) );
    }
}

pub fn run( param:&LearnerParameter ) {
    let mysql_password = match std::env::var("MYSQL_PASSWORD") {
        Ok(val) => format!(":{}", val ),
        Err(_) => String::new(),
    };

    let url = format!("mysql://{}{}@localhost:3306/craft", param.mysql_user, mysql_password );
    eprintln!("Connect to mysql...");
    let mysql_pool_base = Pool::new_manual(2,2,Opts::from_url(&url).unwrap()).unwrap();
    let mysql_pool = Arc::new(Mutex::new(mysql_pool_base));

    // ここから学習のデータ構造作成
    let mut replay_buffer = ReplayBuffer::new(param.replay_buffer_size);

    loop {
        // 理由が分からないですが、このあたりをループの中に入れてあげると実行速度を維持できるので、現状このようにしています。
        // なお影響力が大きいのがoptimizerです。optimizerを外に出すとめちゃくちゃ遅くなります。
        // vsとかnetとかはそんなに影響ないようです。僅かに遅くなってはいるようですが。
        // ↓↓↓ここまで
        let mut vs = nn::VarStore::new(Device::Cpu);
        let net = TchNetwork::new(&vs.root());

        match std::path::Path::new("weights").exists() {
            true => { vs.load("weights").unwrap(); eprintln!("load weights"); }
            false => { eprintln!("cannot find path"); },
        }

        let adam_opt = nn::Adam { wd:0.0001, ..nn::Adam::default() };
        let mut optimizer = adam_opt.build(&vs, 1e-3).unwrap();
        // ↑↑↑ここまで

        run_epoch_loop( &mysql_pool, &mut replay_buffer, &mut optimizer, &vs, &net, param.epochs_per_write );
    }
}
