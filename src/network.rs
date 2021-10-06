use std::error::Error;

use super::logic::{State,Setting,Condition,ACTION_NUM};
use super::mcts::{ActionVector};

use std::fs::File;
use std::io::Read;

use tensorflow::ImportGraphDefOptions;
use tensorflow::Graph;
use tensorflow::Session;
use tensorflow::SessionOptions;
use tensorflow::SessionRunArgs;
use tensorflow::Tensor;
use tensorflow::Operation;

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
pub fn encode_state( s:&State, setting:&Setting ) -> StateVector {
    [
        s.turn as f32 / 128.0,
        s.time as f32 / 256.0,
        s.completed.to_onehot(), // 要らない気がする
        s.working as f32 / setting.max_working as f32,
        s.quality as f32 / setting.max_quality as f32,
        s.durability as f32 / setting.max_durability as f32,
        s.cp as f32 / setting.max_cp as f32,

        if s.inner_quiet == 0 { 0.0 } else { (s.inner_quiet - 1) as f32 / 10.0 },
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
        s.elements as f32 / 10.0,
        s.elements.to_onehot(),
        s.final_appraisal as f32 / 10.0,
        s.final_appraisal.to_onehot(),
        s.muscle_memory as f32 / 10.0,
        s.muscle_memory.to_onehot(),
        s.manipulation as f32 / 10.0,
        s.manipulation.to_onehot(),

        s.elements_used.to_onehot(),
        s.combo_touch.to_onehot(),
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
// ※効率はあんまりよくないです(copyじゃなくて直接配列に書くほうがよいです)
pub fn encode_state_batch( states:&[State], setting:&Setting ) -> tensorflow::Result<Tensor<f32>> {

    let mut state_vec = vec!{};
    state_vec.resize( states.len() * STATE_NUM, 0.0 );

    for i in 0..states.len() {
        state_vec[i*STATE_NUM..(i+1)*STATE_NUM].copy_from_slice( &encode_state(&states[i],setting) );
    }

    Tensor::new(&[states.len() as u64,STATE_NUM as u64]).with_values(&state_vec)
}

fn convert_to_policy_vector( t:&Tensor<f32>, offset:usize ) -> ActionVector {
    let mut res = [0.0;ACTION_NUM];
    for i in 0..ACTION_NUM {
        res[i] = t[i+offset*ACTION_NUM];
    }
    res
}

pub struct Network {
    session : Session,
    op_input : Operation,
    op_policy : Operation,
    op_value : Operation,
}

pub fn load_graph_from_file( filename:&str ) -> Result<Graph, Box<dyn Error>> {
    let mut proto = Vec::new();
    File::open(filename)?.read_to_end(&mut proto)?;

    let mut graph = Graph::new();
    graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;

    Ok(graph)
}

impl Network {
    pub fn load_graph( graph:&Graph ) -> Result<Network, Box<dyn Error>> {

        let session = Session::new(&SessionOptions::new(), &graph)?;
        let op_input = graph.operation_by_name_required("frozen_graph_input")?;
        let op_policy = graph.operation_by_name_required("Identity")?;
        let op_value = graph.operation_by_name_required("Identity_1")?;

        Ok(Network { session:session, op_input:op_input, op_policy:op_policy, op_value:op_value })
    }

    pub fn predict(&self, state:&State, setting:&Setting) -> Result<(ActionVector,f32), Box<dyn Error>> {

        // 入力変数の作成
        let state_vec = encode_state(&state,&setting);
        let state_vec_t = Tensor::new(&[1,STATE_NUM as u64]).with_values(&state_vec)?;

        // 推論
        let mut args = SessionRunArgs::new();
        args.add_feed(&self.op_input, 0, &state_vec_t);
        let policy = args.request_fetch(&self.op_policy, 0);
        let value = args.request_fetch(&self.op_value, 0);
        self.session.run(&mut args)?;

        // 結果の取得
        let policy_res_t:Tensor<f32> = args.fetch(policy)?;
        let value_res_t:Tensor<f32> = args.fetch(value)?;

        Ok((convert_to_policy_vector(&policy_res_t,0),value_res_t[0]))
    }

    pub fn predict_batch(&self, states:&[State], setting:&Setting) -> Result<Vec<(ActionVector,f32)>, Box<dyn Error>> {

        // 入力変数の作成
        let state_vec_t = encode_state_batch( states, setting )?;

        // 推論
        let mut args = SessionRunArgs::new();
        args.add_feed(&self.op_input, 0, &state_vec_t);
        let policy = args.request_fetch(&self.op_policy, 0);
        let value = args.request_fetch(&self.op_value, 0);
        self.session.run(&mut args)?;

        // 結果の取得
        let policy_res_t:Tensor<f32> = args.fetch(policy)?;
        let value_res_t:Tensor<f32> = args.fetch(value)?;

        let policy_iter = (0..states.len()).into_iter().map(|i| convert_to_policy_vector(&policy_res_t,i));
        let value_iter = (0..states.len()).into_iter().map(|i| value_res_t[i]);

        Ok(policy_iter.zip(value_iter).collect())
    }
}
