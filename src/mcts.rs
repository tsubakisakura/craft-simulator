use std::collections::HashMap;
use super::logic::{State,Action,Modifier,Setting,ACTION_NUM};
use super::predictor::*;
use num::FromPrimitive;
use xorshift::{Rng,Xorshift128};
use rand::prelude::*;
use rand::distributions::Dirichlet;

pub type ActionVector = [f32;ACTION_NUM];

#[allow(non_snake_case)]
#[derive(Debug)]
struct Node
{
    // 探索回数
    N : ActionVector,

    // ポリシーネットワークの値
    P : ActionVector,

    // 各アクションを取ったときの、子ノードの評価値の総和
    W : ActionVector,
}

pub struct MCTSContext
{
    // ディリクレノイズの為のパラメータ。
    // ディリクレノイズはこの投稿を参考
    // https://tadaoyamaoka.hatenablog.com/entry/2017/12/10/230549
    alpha: f32,

    // ディリクレノイズの割合のパラメータ。
    // 1に近づくほどノイズの割合が大きくなります。0の時はノイズなしで探索されます。
    eps: f32,

    // UCTの定数
    c_puct: f32,

    // ノード一覧
    nodes: HashMap<State,Node>,

    // 予測システム
    predict_queue: PredictQueue,

    // 本コンテキストでキューに渡すグラフ名
    graph_filename: String,
}

enum SearchResult {
    Expand(State), // 途中の場合
    Reward(f32),   // 報酬がもらえる場合
}

impl State {
    // 現実にあり得ないパターンを除外します。
    // 初手インナークワイエット使うくらいなら真価を使うとか、そういう基本的な手だけ対策します。
    // あと「作業で辿りつける場合は最終確認は無効」とかも削ってよいかもしれません。
    fn check_action_ex(&self, a:&Action) -> bool {
        if self.turn == 1 {
            // 1ターン目は確信か真価に限定します。
            // 流石にこれ以外のスタートパターンは現実的に存在しないため、これだけは無視します
            match a {
                Action::MuscleMemory => true,
                Action::Reflect => true,
                _ => false,
            }
        }
        else if self.final_appraisal > 0 && *a == Action::FinalAppraisal {
            // もし最終確認が有効な場合、最終確認を新たに使うことはあり得ません。これは何の役にも立たずCPだけを消費します。
            false
        }
        else if self.working < 5000 && *a == Action::FinalAppraisal {
            // もし１手で完成に辿りつけない作業工数である場合、最終確認を使うことはあり得ません。次のターンで使えば良いためです。
            false
        }
        else {
            self.check_action(a)
        }
    }
}

#[allow(non_snake_case)]
fn get_scores(c_puct:f32, s:&State, node:&Node) -> ActionVector {
    let mut scores = [0.0;ACTION_NUM];

    let sum_N : f32 = node.N.iter().sum();
    let sum_N_sqrt = sum_N.sqrt();

    for a in 0..ACTION_NUM {
        if s.check_action_ex(&Action::from_usize(a).unwrap()) {
            let U = c_puct * node.P[a] * sum_N_sqrt / (1.0+node.N[a]);
            let Q = if node.N[a] != 0.0 { node.W[a] / node.N[a] } else { 0.0 };
            scores[a] = U+Q;
        }
        else {
            scores[a] = f32::NEG_INFINITY;
        }
    }

    scores
}

fn get_mcts_policy( v:&ActionVector ) -> ActionVector {
    let sum : f32 = v.iter().sum();
    let mut r = v.clone();
    r.iter_mut().for_each(|x| *x /= sum);
    r
}

// [a,b]区間でcがどの位置にいるかを取得します。
fn lerp_clip( a:f32, b:f32, c:f32 ) -> f32 {
    let t = (c-a)/(b-a);
    t.max(0.0).min(1.0)
}

// 報酬関数です。
pub fn get_reward(s:&State,setting:&Setting) -> f32 {
    if s.is_destroyed() {
        0.0
    }
    else {
        // 品質とターンボーナスのマージ用定数
        let t = 0.9;

        // 品質[0,1]
        let q = s.quality as f32 / setting.max_quality as f32;

        // ターンボーナス[0,1]
        //let b = lerp_clip(60.0,20.0,s.turn as f32) * lerp_clip(81000.0,81400.0,s.quality as f32);
        let b = if s.quality == setting.max_quality { lerp_clip(60.0,20.0,s.turn as f32) } else { 0.0 };

        // 結果はqとbを適当にマージして決めます
        t*q + (1.0-t)*b
    }
}

fn select_max_indices(mcts_policy:&ActionVector) -> Vec<usize> {
    // Rustでf32やf64の配列の最大値を得る方法
    // https://qiita.com/lo48576/items/343ca40a03c3b86b67cb
    let max_value = mcts_policy.iter().fold(0.0/0.0, |m, v| v.max(m));

    // enumerateでインデックスを付けて、最大要素でフィルタして、第１要素を抜き出します
    mcts_policy.iter().enumerate().filter(|(_,&v)| v == max_value).map(|(i,_)| i).collect()
}

#[test]
fn test_select_max_indices()
{
    let mut mcts_policy = [0.0;ACTION_NUM];
    mcts_policy[0] = 1.0;
    mcts_policy[15] = 1.0;
    assert_eq!( vec![0,15], select_max_indices(&mcts_policy) );
}

fn choose_max_index(mcts_policy:&ActionVector, rng:&mut Xorshift128) -> usize {
    let indices = select_max_indices(&mcts_policy);
    *rng.choose(&indices).unwrap()
}

// ノードの選択確率の通りに選択します。
// mtct_policyは総和が1.0である必要があります。
pub fn select_action_weighted(mcts_policy:&ActionVector, rng:&mut Xorshift128) -> Action {
    // 数値誤差により全アクションの確率を総和してもランダム値がどのアクションにも該当しない場合があります。
    // 失敗した場合はもう一度選択します
    loop {
        let x = rng.next_f32();
        let mut sum = 0.0;

        for i in 0..ACTION_NUM {
            sum += mcts_policy[i];
            if x < sum {
                return Action::from_usize( i ).unwrap()
            }
        }
    }
}

// greedy(一番よいやつ)を選択します
pub fn select_action_greedy(mcts_policy:&ActionVector, rng:&mut Xorshift128) -> Action {
    Action::from_usize( choose_max_index(&mcts_policy, rng) ).unwrap()
}

impl MCTSContext {

    pub fn new( c_puct:f32, alpha:f32, eps:f32, predict_queue:PredictQueue, graph_filename:String ) -> MCTSContext {
        MCTSContext {
            c_puct: c_puct,
            alpha: alpha,
            eps: eps,
            nodes: HashMap::new(),
            predict_queue: predict_queue,
            graph_filename: graph_filename,
        }
    }

    #[allow(non_snake_case)]
    fn add_dirichlet_noise(&mut self, s:&State, _modifier:&mut Modifier) {
        if self.eps > 0.0 {
            // ノードを探し出します。expandしてますので絶対に成功します。
            let mut node = self.nodes.get_mut(s).unwrap();

            // ディリクレノイズを計算します。
            // まず合法手のインデックスだけ求めます
            let mut valid_actions : Vec<usize> = vec!{};

            for a in 0..ACTION_NUM {
                if s.check_action_ex(&Action::from_usize(a).unwrap()) {
                    valid_actions.push(a);
                }
            }

            // ディリクレ分布を求めます
            let dirichlet = Dirichlet::new_with_param(self.alpha as f64, valid_actions.len());
            let samples = dirichlet.sample(&mut rand::thread_rng()); // TODO: Xorshiftが使えなかった

            // ノイズを対象インデックスに足す
            for i in 0..valid_actions.len() {
                node.P[valid_actions[i]] = (1.0-self.eps) * node.P[valid_actions[i]] + self.eps * samples[i] as f32;
            }
        }
    }

    // 現在の地点から葉までノードを探索します。
    fn search_leaf(&self, start:&State, modifier:&mut Modifier) -> (Vec<(State,usize)>,SearchResult) {
        let mut s = start.clone();
        let mut path = vec!{};
        loop {
            if s.is_terminated() {
                return (path,SearchResult::Reward(get_reward(&s,&modifier.setting)));
            }
            else if let Some(node) = self.nodes.get(&s) {
                let scores = get_scores(self.c_puct, &s, node);
                let a = choose_max_index(&scores, &mut modifier.rng);
                let ns = s.run_action(modifier, &Action::from_usize(a).unwrap());
                path.push((s,a));
                s = ns
            }
            else {
                return (path,SearchResult::Expand(s));
            }
        }
    }

    // ノードを展開します。
    fn expand(&mut self, s:State, nn_policy:ActionVector) {
        // insert関数はOption<V>で元の値を返しますが、expandの時点では元のノードが存在しないため、常にNoneが帰ります
        self.nodes.insert(s, Node {
            N: [0.0;ACTION_NUM],
            W: [0.0;ACTION_NUM],
            P: nn_policy,
        });
    }

    // 評価値を足します。
    fn add_value(&mut self, path:&Vec<(State,usize)>, v:f32) {
        for (s,a) in path {
            let node = self.nodes.get_mut(s).unwrap();
            node.W[*a] += v;
            node.N[*a] += 1.0;
        }
    }

    async fn run_simulation(&mut self, start:&State, modifier:&mut Modifier) {
        let ret = self.search_leaf(start,modifier);
        match ret {
            (path,SearchResult::Expand(leaf)) => {
                let (nn_policy,nn_value) = self.predict_queue.async_predict(self.graph_filename.clone(), leaf.clone()).await;
                self.expand(leaf,nn_policy);
                self.add_value(&path,nn_value);
            },
            (path,SearchResult::Reward(reward)) => {
                self.add_value(&path,reward);
            },
        }
    }

    // 現在の状態に絶対に辿りつけないノードを除去します。
    //
    // 設計変更や最終確認が同一ターンで別状態となるため同一ターンは維持しています。
    // 上記ルールも判定したうえで消せばメモリ効率が上がりますが、そこまで切り詰める必要もないので、このルールで保留しています
    fn remove_unused_nodes(&mut self, root_state:&State ) {
        self.nodes.retain(|s,_| s.turn >= root_state.turn)
    }

    pub async fn search(&mut self, s:&State, modifier:&mut Modifier, num_simulations:u32) -> ActionVector {

        self.remove_unused_nodes(s);

        if !self.nodes.contains_key( s ) {
            let (nn_policy,_) = self.predict_queue.async_predict(self.graph_filename.clone(), s.clone()).await;
            self.expand( s.clone(), nn_policy );
        }

        // 初手の場合だけディリクレノイズを加えます。
        self.add_dirichlet_noise(s, modifier);

        // シミュレーションを規定回数実行します
        for _ in 0..num_simulations {
            self.run_simulation(s,modifier).await;
        }

        // 方策決定します。単に全体をNで割って返す
        get_mcts_policy( &self.nodes.get(s).unwrap().N )
    }

    // デバッグする時に呼び出すコードなので無効にしておきます
    #[allow(dead_code)]
    pub fn print_stats(&self) {
        eprintln!("Node num:{}", self.nodes.len());
    }
}

// デバッグする時に呼び出すコードなので無効にしておきます
#[allow(dead_code)]
pub fn print_mcts_stats() {
    eprintln!("State size:{}", std::mem::size_of::<State>());
    eprintln!("Node size:{}", std::mem::size_of::<Node>());
}
