
use super::selfplay::{Sample,Record};
use super::setting::ModifierParameter;
use super::network::encode_state;

pub trait Formatter {
    fn format(&self, record:&Record) -> Vec<String>;
}

#[derive(Clone)]
pub struct TsvFormatter {
    pub mod_param : ModifierParameter,
}

fn export_by_tsv(s:&Sample, mod_param:&ModifierParameter, reward:f32) -> String {
    let state_vec = encode_state(&s.state, mod_param);
    let reward_vec = [reward];

    // State -> Policy -> Value の順に並べます
    let iter = state_vec.iter().chain(s.mcts_policy.iter()).chain(reward_vec.iter());

    // 文字列化
    let dst : Vec<String> = iter.map(|x| format!("{:.8}",x)).collect();

    // TSVにする
    dst.join("\t")
}

impl Formatter for TsvFormatter {
    fn format(&self, record:&Record) -> Vec<String> {
        record.samples.iter().map(|x| export_by_tsv(&x, &self.mod_param, record.reward)).collect()
    }
}
