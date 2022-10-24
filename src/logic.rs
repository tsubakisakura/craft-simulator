
extern crate xorshift;

use super::setting::ModifierParameter;
use serde::{Serialize,Deserialize};
use std::cmp::min;
use std::hash::Hash;
use xorshift::{Rng,Xorshift128};
use num::traits::{FromPrimitive,ToPrimitive};

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Hash)]
pub enum Condition
{
    Standard,       // 通常
    HighQuality,    // 高品質
    HighProgress,   // 高進捗
    HighEfficiency, // 高能率
    HighSustain,    // 高持続
    Solid,          // 頑丈
    Stable,         // 安定
}

pub const ACTION_NUM: usize = 32;

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub enum Action {
    BasicSynthesis,     // 作業
    BasicTouch,         // 加工
    MastersMend,        // マスターズメンド
    HastyTouch,         // ヘイスティタッチ
    RapidSynthesis,     // 突貫作業
    Observe,            // 経過観察
    TricksOfTheTrade,   // 秘訣
    WasteNot,           // 倹約
    Veneration,         // ヴェネレーション
    StandardTouch,      // 中級加工
    GreatStrides,       // グレートストライド
    Innovation,         // イノベーション
    FinalAppraisal,     // 最終確認
    WasteNot2,          // 長期倹約
    ByregotsBlessing,   // ビエルゴの祝福
    PreciseTouch,       // 集中加工
    MuscleMemory,       // 確信
    CarefulObservation, // 設計変更
    CarefulSynthesis,   // 模範作業
    Manipulation,       // マニピュレーション
    PrudentTouch,       // 倹約加工
    FocusedSynthesis,   // 注視作業
    FocusedTouch,       // 注視加工
    Reflect,            // 真価
    PreparatoryTouch,   // 下地加工
    Groundwork,         // 下地作業
    DelicateSynthesis,  // 精密作業
    IntensiveSynthesis, // 集中作業
    AdvancedTouch,      // 上級加工
    HeartAndSoul,       // 一心不乱
    PrudentSynthesis,   // 倹約作業
    TrainedFinesse,     // 匠の神業
}

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq,Hash)]
pub struct State
{
    pub turn : u32,                   // ターン
    pub time : u32,                   // 推定経過時間
    pub completed: bool,              // 完成フラグ
    pub working : u32,                // 工数
    pub quality: u32,                 // 品質
    pub durability : u32,             // 耐久
    pub cp : u32,                     // CP
    pub inner_quiet : u32,            // インナークワイエットスタック
    pub careful_observation : u32,    // 残設計変更
    pub waste_not : u32,              // 残倹約
    pub veneration : u32,             // 残ヴェネレーション
    pub great_strides : u32,          // 残グレートストライド
    pub innovation : u32,             // 残イノベーション
    pub final_appraisal : u32,        // 残最終確認
    pub muscle_memory : u32,          // 残確信
    pub manipulation : u32,           // マニピュレーション
    pub heart_and_soul : bool,        // 一心不乱有効
    pub heart_and_soul_used : bool,   // 一心不乱を使用済みかどうか
    pub combo_basic_touch : bool,     // 直前に加工したかどうか
    pub combo_standard_touch : bool,  // 直前に中級加工したかどうか
    pub combo_observe : bool,         // 直前に経過観察したかどうか
    pub condition : Condition         // 状態
}

pub struct Modifier
{
    pub mod_param : ModifierParameter,
    pub rng : Xorshift128,
}

// https://totem3.hatenablog.jp/entry/2015/08/07/222303
// enumと数値型の変換は原始的なこの方法で変換してみます。
// 他に旨い手があるのかもしれないですが
impl FromPrimitive for Action {
    fn from_i64(n:i64) -> Option<Action> {
        if 0 <= n && n <= 999 {
            Self::from_u64(n as u64)
        }
        else {
            None
        }
    }

    fn from_u64(n:u64) -> Option<Action> {
        match n {
            0 => Some(Action::BasicSynthesis),     // 作業
            1 => Some(Action::BasicTouch),         // 加工
            2 => Some(Action::MastersMend),        // マスターズメンド
            3 => Some(Action::HastyTouch),         // ヘイスティタッチ
            4 => Some(Action::RapidSynthesis),     // 突貫作業
            5 => Some(Action::Observe),            // 経過観察
            6 => Some(Action::TricksOfTheTrade),   // 秘訣
            7 => Some(Action::WasteNot),           // 倹約
            8 => Some(Action::Veneration),         // ヴェネレーション
            9 => Some(Action::StandardTouch),      // 中級加工
            10 => Some(Action::GreatStrides),       // グレートストライド
            11 => Some(Action::Innovation),         // イノベーション
            12 => Some(Action::FinalAppraisal),     // 最終確認
            13 => Some(Action::WasteNot2),          // 長期倹約
            14 => Some(Action::ByregotsBlessing),   // ビエルゴの祝福
            15 => Some(Action::PreciseTouch),       // 集中加工
            16 => Some(Action::MuscleMemory),       // 確信
            17 => Some(Action::CarefulObservation), // 設計変更
            18 => Some(Action::CarefulSynthesis),   // 模範作業
            19 => Some(Action::Manipulation),       // マニピュレーション
            20 => Some(Action::PrudentTouch),       // 倹約加工
            21 => Some(Action::FocusedSynthesis),   // 注視作業
            22 => Some(Action::FocusedTouch),       // 注視加工
            23 => Some(Action::Reflect),            // 真価
            24 => Some(Action::PreparatoryTouch),   // 下地加工
            25 => Some(Action::Groundwork),         // 下地作業
            26 => Some(Action::DelicateSynthesis),  // 精密作業
            27 => Some(Action::IntensiveSynthesis), // 集中作業
            28 => Some(Action::AdvancedTouch),      // 上級加工
            29 => Some(Action::HeartAndSoul),       // 一心不乱
            30 => Some(Action::PrudentSynthesis),   // 倹約作業
            31 => Some(Action::TrainedFinesse),     // 匠の神業
            _ => None
        }
    }
}

impl ToPrimitive for Action {
    fn to_i64(&self) -> Option<i64> {
        Some(self.to_u64().unwrap() as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some( match *self {
            Action::BasicSynthesis => 0,
            Action::BasicTouch => 1,
            Action::MastersMend => 2,
            Action::HastyTouch => 3,
            Action::RapidSynthesis => 4,
            Action::Observe => 5,
            Action::TricksOfTheTrade => 6,
            Action::WasteNot => 7,
            Action::Veneration => 8,
            Action::StandardTouch => 9,
            Action::GreatStrides => 10,
            Action::Innovation => 11,
            Action::FinalAppraisal => 12,
            Action::WasteNot2 => 13,
            Action::ByregotsBlessing => 14,
            Action::PreciseTouch => 15,
            Action::MuscleMemory => 16,
            Action::CarefulObservation => 17,
            Action::CarefulSynthesis => 18,
            Action::Manipulation => 19,
            Action::PrudentTouch => 20,
            Action::FocusedSynthesis => 21,
            Action::FocusedTouch => 22,
            Action::Reflect => 23,
            Action::PreparatoryTouch => 24,
            Action::Groundwork => 25,
            Action::DelicateSynthesis => 26,
            Action::IntensiveSynthesis => 27,
            Action::AdvancedTouch => 28,
            Action::HeartAndSoul => 29,
            Action::PrudentSynthesis => 30,
            Action::TrainedFinesse => 31,
        })
    }
}

impl Modifier {
    fn try_random(&mut self, success_rate : f32) -> bool {
        self.rng.next_f32() < success_rate
    }
}

fn decrement_clip( x : u32 ) -> u32 {
    if x > 0 { x - 1 } else { 0 }
}

impl State {
    pub fn new(mod_param:&ModifierParameter) -> Self {
        State {
            turn:1,
            time:0,
            completed:false,
            working:0,
            quality:0,
            durability:mod_param.max_durability,
            cp:mod_param.max_cp,
            inner_quiet:0,
            careful_observation:3,
            waste_not:0,
            veneration:0,
            great_strides:0,
            innovation:0,
            final_appraisal:0,
            muscle_memory:0,
            manipulation:0,
            heart_and_soul:false,
            heart_and_soul_used:false,
            combo_basic_touch:false,
            combo_standard_touch:false,
            combo_observe:false,
            condition:Condition::Standard,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        !self.completed && self.durability <= 0
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn is_terminated(&self) -> bool {
        self.is_completed() || self.is_destroyed()
    }

    // 必要CP一覧
    // CPはStateに依存した関数であるためStateの関数とします
    pub fn get_required_cp(&self, a:&Action) -> u32 {
        let required_cp = match a {
            Action::BasicSynthesis => 0,
            Action::BasicTouch => 18,
            Action::MastersMend => 88,
            Action::HastyTouch => 0,
            Action::RapidSynthesis => 0,
            Action::Observe => 7,
            Action::TricksOfTheTrade => 0,
            Action::WasteNot => 56,
            Action::Veneration => 18,
            Action::StandardTouch => if self.combo_basic_touch { 18 } else { 32 },
            Action::GreatStrides => 32,
            Action::Innovation => 18,
            Action::FinalAppraisal => 1,
            Action::WasteNot2 => 98,
            Action::ByregotsBlessing => 24,
            Action::PreciseTouch => 18,
            Action::MuscleMemory => 6,
            Action::CarefulObservation => 0,
            Action::CarefulSynthesis => 7,
            Action::Manipulation => 96,
            Action::PrudentTouch => 25,
            Action::FocusedSynthesis => 5,
            Action::FocusedTouch => 18,
            Action::Reflect => 6,
            Action::PreparatoryTouch => 40,
            Action::Groundwork => 18,
            Action::DelicateSynthesis => 32,
            Action::IntensiveSynthesis => 6,
            Action::AdvancedTouch => if self.combo_standard_touch { 18 } else { 46 },
            Action::HeartAndSoul => 0,
            Action::PrudentSynthesis => 18,
            Action::TrainedFinesse => 32,
        };

        // 高能率の場合は半減しますが端数切り上げなので1足します
        if self.condition == Condition::HighEfficiency {
            (required_cp+1)/2
        }
        else {
            required_cp
        }
    }

    // Velvet Weissmelさんの統計データを参考に設定しています
    // https://jp.finalfantasyxiv.com/lodestone/character/3514261/blog/4645845/
    fn next_condition(rng:&mut Xorshift128) -> Condition {
        let v = rng.next_f32();
        match v {
            _ if v < 0.37 => Condition::Standard,
            _ if v < 0.49 => Condition::HighQuality,
            _ if v < 0.61 => Condition::HighProgress,
            _ if v < 0.73 => Condition::Stable,
            _ if v < 0.85 => Condition::HighSustain,
            _             => Condition::Solid,
        }
    }

    fn change_condition(&self,modifier:&mut Modifier) -> State {
        if self.is_terminated() {
            self.clone()
        }
        else {
            State { condition: State::next_condition(&mut modifier.rng), .. *self }
        }
    }

    fn next_turn(&self,modifier:&mut Modifier) -> State {
        State {
            turn: if self.is_terminated() { self.turn } else { self.turn + 1 },
            durability: if self.manipulation == 0 || self.is_terminated() { self.durability } else { min(self.durability+5,modifier.mod_param.max_durability) },
            veneration: decrement_clip(self.veneration),
            waste_not: decrement_clip(self.waste_not),
            great_strides: decrement_clip(self.great_strides),
            innovation: decrement_clip(self.innovation),
            final_appraisal: decrement_clip(self.final_appraisal),
            muscle_memory: decrement_clip(self.muscle_memory),
            manipulation: decrement_clip(self.manipulation),
            combo_basic_touch: false,
            combo_standard_touch: false,
            combo_observe: false,
            .. *self
        }
    }

    fn duration(&self,x:u32) -> u32 {
        if self.condition == Condition::HighSustain { x+2 } else { x }
    }

    fn probability(&self,x:f32) -> f32 {
        if self.condition == Condition::Stable { x+0.25 } else { x }
    }

    // 作業に対する品質報酬
    // 情報が無いので、そのまま決め打ちで打ち込んでます
    fn working_reward(&self, mod_param:&ModifierParameter, efficiency : f64 ) -> u32 {
        // 情報が無いのでそのまま決め打ちの数値の対応です。それ以外に対応することになったらやる
        if mod_param.work_accuracy != 2769 {
            return 99999;
        }

        let q = 472.0;
        let cond_rate = if self.condition == Condition::HighProgress { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if self.veneration > 0 { 0.5 } else { 0.0 } + if self.muscle_memory > 0 { 1.0 } else { 0.0 };

        return ( q * cond_rate * efficiency * buff_rate ) as u32;
    }

    // 効率に対する品質報酬
    // こちらの記事が紹介しているcalculatorの内容を参考にしています。
    // https://jp.finalfantasyxiv.com/lodestone/character/29523439/blog/4641394/
    // 完全一致はしませんが、近似値として使えます。完全一致を求めるならば、データシートを作るほうが良いと思う
    fn quality_reward(&self, mod_param:&ModifierParameter, efficiency : u32) -> u32 {
        let inner_quiet : f64 = From::from(self.inner_quiet);
        let process_accuracy : f64 = From::from(mod_param.process_accuracy);
        let required_process_accuracy : f64 = From::from(mod_param.required_process_accuracy);

        let f = process_accuracy + process_accuracy * (inner_quiet * 20.0 / 100.0);
        let q1 = f*35.0/100.0 + 35.0;
        let q2 = q1 * (f + 10000.0) / (required_process_accuracy + 10000.0);
        let q3 = q2 * 60.0 / 100.0;
        let cond_rate = if self.condition == Condition::HighQuality { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if self.great_strides > 0 { 1.0 } else { 0.0 } + if self.innovation > 0 { 0.5 } else { 0.0 };

        return ( q3 * cond_rate * efficiency as f64 * buff_rate ) as u32 / 100;
    }

    fn add_working(&self, mod_param:&ModifierParameter, efficiency : f64) -> State {
        let w = self.working + self.working_reward(&mod_param,efficiency);

        if w >= mod_param.max_working {
            if self.final_appraisal > 0 {
                State { working:mod_param.max_working - 1, muscle_memory:0, final_appraisal:0, .. *self } // 最終確認バフを消して完了直前に設定
            }
            else {
                State { working:mod_param.max_working, muscle_memory:0, completed: true, .. *self } // 作業完了
            }
        }
        else {
            State { working:w, muscle_memory:0, .. *self } // 未完なので足すだけ
        }
    }

    fn add_quality_base(&self, mod_param:&ModifierParameter, efficiency:u32) -> State {
        State {
            quality: min(self.quality + self.quality_reward(&mod_param,efficiency), mod_param.max_quality),
            great_strides: 0,
            .. *self
        }
    }

    fn add_inner_quiet(&self, inner_quiet_stack:u32) -> State {
        State {
            inner_quiet: min(self.inner_quiet+inner_quiet_stack,10),
            .. *self
        }
    }

    fn add_time(&self, x:u32) -> State {
        State { time: self.time + x, .. * self }
    }

    fn set_inner_quiet(&self, inner_quiet:u32) -> State {
        State { inner_quiet: inner_quiet, .. *self }
    }

    fn add_quality(&self, mod_param:&ModifierParameter, efficiency:u32, inner_quiet_stack:u32) -> State {
        self.add_quality_base(&mod_param,efficiency).add_inner_quiet(inner_quiet_stack)
    }

    fn add_quality_byregots(&self, mod_param:&ModifierParameter) -> State {
        self.add_quality_base(&mod_param,100 + self.inner_quiet * 20).set_inner_quiet(0)
    }

    fn consume_careful_observation(&self) -> State {
        State { careful_observation: self.careful_observation-1, .. *self }
    }

    fn consume_cp(&self, a:&Action) -> State {
        State { cp:self.cp - self.get_required_cp(a), .. *self }
    }

    fn consume_durability(&self, x:u32) -> State {
        let cond_rate = if self.condition == Condition::Solid { 0.5 } else { 1.0 };
        let waste_not_rate = if self.waste_not > 0 { 0.5 } else { 1.0 };
        let q = (((x as f32) * cond_rate * waste_not_rate).ceil()) as u32;

        State { durability: if self.durability > q { self.durability - q } else { 0 }, .. *self }
    }

    fn add_durability(&self, x:u32, mod_param:&ModifierParameter) -> State {
        State { durability: min(self.durability+x,mod_param.max_durability), ..*self }
    }

    fn add_cp(&self, x:u32, mod_param:&ModifierParameter) -> State {
        State { cp: min(self.cp+x,mod_param.max_cp), ..*self }
    }

    fn clear_manipulation(&self) -> State {
        State { manipulation:0, .. *self }
    }

    fn set_manipulation(&self, dur:u32) -> State {
        State { manipulation: self.duration(dur), .. *self }
    }

    fn set_great_strides(&self, dur:u32) -> State {
        State { great_strides: self.duration(dur), .. *self }
    }

    fn set_innovation(&self, dur:u32) -> State {
        State { innovation: self.duration(dur), .. *self }
    }

    fn set_waste_not(&self, dur:u32) -> State {
        State { waste_not: self.duration(dur), .. *self }
    }

    fn set_veneration(&self, dur:u32) -> State {
        State { veneration: self.duration(dur), .. *self }
    }

    fn set_final_appraisal(&self, dur:u32) -> State {
        State { final_appraisal: self.duration(dur), .. *self }
    }

    fn set_muscle_memory(&self, dur:u32) -> State {
        State { muscle_memory: self.duration(dur), .. *self }
    }

    fn set_heart_and_soul(&self) -> State {
        State { heart_and_soul: true, heart_and_soul_used: true, .. *self }
    }

    fn set_combo_basic_touch(&self) -> State {
        State { combo_basic_touch: true, .. *self }
    }

    fn set_combo_standard_touch(&self, combo_basic_touch:bool) -> State {
        State { combo_standard_touch: combo_basic_touch, .. *self }
    }

    fn set_combo_observe(&self) -> State {
        State { combo_observe: true, .. *self }
    }

    fn clear_combo(&self) -> State {
        State { combo_observe: false, combo_basic_touch: false, combo_standard_touch: false, .. *self }
    }

    fn clear_heart_and_soul(&self) -> State {
        if self.condition == Condition::HighQuality {
            self.clone()
        }
        else {
            State { heart_and_soul: false, .. *self }
        }
    }

    // 実行確認
    pub fn check_action(&self, a:&Action) -> bool {
        if self.cp >= self.get_required_cp(&a) {
            match a {
                Action::TricksOfTheTrade => self.condition == Condition::HighQuality || self.heart_and_soul,
                Action::ByregotsBlessing => self.inner_quiet > 0, // ビエルゴはinner_quiet初期値の時は使えません
                Action::PreciseTouch => self.condition == Condition::HighQuality || self.heart_and_soul,
                Action::MuscleMemory => self.turn == 1, // 確信バフは最終確認で消えません
                Action::CarefulObservation => self.careful_observation > 0,
                Action::PrudentTouch => self.waste_not == 0,
                Action::Reflect => self.turn == 1, // 真価バフは最終確認で消えません
                Action::IntensiveSynthesis => self.condition == Condition::HighQuality || self.heart_and_soul,
                Action::HeartAndSoul => !self.heart_and_soul_used,
                Action::PrudentSynthesis => self.waste_not == 0,
                Action::TrainedFinesse => self.inner_quiet == 10, // 匠の神業はIQ10限定
                _ => true
            }
        }
        else {
            false
        }
    }

    // 作業
    fn action_basic_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.mod_param,1.2).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 加工
    fn action_basic_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.mod_param,100,1).consume_cp(&Action::BasicTouch).consume_durability(10).next_turn(modifier).set_combo_basic_touch().change_condition(modifier).add_time(3)
    }

    // マスターズメンド
    fn action_masters_mend(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::MastersMend).add_durability(30,&modifier.mod_param).next_turn(modifier).change_condition(modifier).add_time(2)
    }

    // ヘイスティタッチ
    fn action_hasty_touch(&self, modifier:&mut Modifier) -> State {
        if modifier.try_random(self.probability(0.5)) {
            // 成功時
            self.add_quality(&modifier.mod_param,100,1).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗時
            self.consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // 突貫作業
    fn action_rapid_synthesis(&self, modifier:&mut Modifier) -> State {
        if modifier.try_random(self.probability(0.5)) {
            // 成功時
            self.add_working(&modifier.mod_param,5.0).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗時
            self.consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // 経過観察
    fn action_observe(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::Observe).next_turn(modifier).set_combo_observe().change_condition(modifier).add_time(3)
    }

    // 秘訣
    fn action_trick_of_the_trade(&self, modifier:&mut Modifier) -> State {
        self.next_turn(modifier).add_cp(20,&modifier.mod_param).clear_heart_and_soul().change_condition(modifier).add_time(3)
    }

    // 倹約
    fn action_waste_not(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::WasteNot).next_turn(modifier).set_waste_not(4).change_condition(modifier).add_time(2)
    }

    // ヴェネレーション
    fn action_veneration(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::Veneration).next_turn(modifier).set_veneration(4).change_condition(modifier).add_time(2)
    }

    // 中級加工
    fn action_standard_touch(&self, modifier:&mut Modifier) -> State {
        // 上級加工へのコンボは直前の中級加工コンボが有効でなければ発動しません
        let combo_basic_touch = self.combo_basic_touch;
        self.add_quality(&modifier.mod_param,125,1).consume_cp(&Action::StandardTouch).consume_durability(10).next_turn(modifier).set_combo_standard_touch(combo_basic_touch).change_condition(modifier).add_time(3)
    }

    // グレートストライド
    fn action_great_strides(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::GreatStrides).next_turn(modifier).set_great_strides(3).change_condition(modifier).add_time(2)
    }

    // イノベーション
    fn action_innovation(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::Innovation).next_turn(modifier).set_innovation(4).change_condition(modifier).add_time(2)
    }

    // 最終確認
    fn action_final_apprisal(&self, _modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::FinalAppraisal).clear_combo().set_final_appraisal(5).add_time(2)
    }

    // 長期倹約
    fn action_waste_not2(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::WasteNot2).next_turn(modifier).set_waste_not(8).change_condition(modifier).add_time(2)
    }

    // ビエルゴの祝福
    fn action_byregots_blessing(&self, modifier:&mut Modifier) -> State {
        self.add_quality_byregots(&modifier.mod_param).consume_cp(&Action::ByregotsBlessing).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 集中加工
    fn action_precise_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.mod_param,150,2).consume_cp(&Action::PreciseTouch).consume_durability(10).next_turn(modifier).clear_heart_and_soul().change_condition(modifier).add_time(3)
    }

    // 確信
    fn action_muscle_memory(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.mod_param,3.0).consume_cp(&Action::MuscleMemory).consume_durability(10).next_turn(modifier).set_muscle_memory(5).change_condition(modifier).add_time(3)
    }

    // 設計変更
    fn action_careful_observation(&self, modifier:&mut Modifier) -> State {
        self.clear_combo().consume_careful_observation().change_condition(modifier).add_time(2)
    }

    // 模範作業
    fn action_careful_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.mod_param,1.8).consume_cp(&Action::CarefulSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // マニピュレーション
    fn action_manipulation(&self, modifier:&mut Modifier) -> State {
        self.clear_manipulation().consume_cp(&Action::Manipulation).next_turn(modifier).set_manipulation(8).change_condition(modifier).add_time(2)
    }

    // 倹約加工
    fn action_prudent_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.mod_param,100,1).consume_cp(&Action::PrudentTouch).consume_durability(5).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 注視作業
    fn action_focused_synthesis(&self, modifier:&mut Modifier) -> State {
        if self.combo_observe || modifier.try_random(self.probability(0.5)) {
            // 成功の場合
            self.add_working(&modifier.mod_param,1.5).consume_cp(&Action::FocusedSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗の場合
            self.consume_cp(&Action::FocusedSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // 注視作業
    fn action_focused_touch(&self, modifier:&mut Modifier) -> State {
        if self.combo_observe || modifier.try_random(self.probability(0.5)) {
            // 成功の場合
            self.add_quality(&modifier.mod_param,150,1).consume_cp(&Action::FocusedTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗の場合
            self.consume_cp(&Action::FocusedTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // 真価
    fn action_reflect(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.mod_param,100,2).consume_cp(&Action::Reflect).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 下地加工
    fn action_preparatory_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.mod_param,200,2).consume_cp(&Action::PreparatoryTouch).consume_durability(20).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 下地作業
    fn action_groundwork(&self, modifier:&mut Modifier) -> State {
        let efficiency = if self.get_required_cp(&Action::Groundwork) < self.durability as u32 { 1.8 } else { 3.6 };

        self.add_working(&modifier.mod_param,efficiency).consume_cp(&Action::Groundwork).consume_durability(20).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 精密作業
    fn action_delecate_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.mod_param,1.0).add_quality(&modifier.mod_param,100,1).consume_cp(&Action::DelicateSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 集中作業
    fn action_intensive_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.mod_param,4.0).consume_cp(&Action::IntensiveSynthesis).consume_durability(10).next_turn(modifier).clear_heart_and_soul().change_condition(modifier).add_time(3)
    }

    // 上級加工
    fn action_advanced_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.mod_param,150,1).consume_cp(&Action::AdvancedTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 一心不乱
    fn action_heart_and_soul(&self, _modifier:&mut Modifier) -> State {
        self.clear_combo().set_heart_and_soul().add_time(2)
    }

    // 倹約作業
    fn action_prudent_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.mod_param,1.8).consume_cp(&Action::PrudentSynthesis).consume_durability(5).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 匠の神業
    fn action_trained_finesse(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.mod_param,100,1).consume_cp(&Action::TrainedFinesse).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // アクション取得
    pub fn run_action(&self, modifier:&mut Modifier, a:&Action) -> State {
        match a {
            Action::BasicSynthesis => self.action_basic_synthesis(modifier),
            Action::BasicTouch => self.action_basic_touch(modifier),
            Action::MastersMend => self.action_masters_mend(modifier),
            Action::HastyTouch => self.action_hasty_touch(modifier),
            Action::RapidSynthesis => self.action_rapid_synthesis(modifier),
            Action::Observe => self.action_observe(modifier),
            Action::TricksOfTheTrade => self.action_trick_of_the_trade(modifier),
            Action::WasteNot => self.action_waste_not(modifier),
            Action::Veneration => self.action_veneration(modifier),
            Action::StandardTouch => self.action_standard_touch(modifier),
            Action::GreatStrides => self.action_great_strides(modifier),
            Action::Innovation => self.action_innovation(modifier),
            Action::FinalAppraisal => self.action_final_apprisal(modifier),
            Action::WasteNot2 => self.action_waste_not2(modifier),
            Action::ByregotsBlessing => self.action_byregots_blessing(modifier),
            Action::PreciseTouch => self.action_precise_touch(modifier),
            Action::MuscleMemory => self.action_muscle_memory(modifier),
            Action::CarefulObservation => self.action_careful_observation(modifier),
            Action::CarefulSynthesis => self.action_careful_synthesis(modifier),
            Action::Manipulation => self.action_manipulation(modifier),
            Action::PrudentTouch => self.action_prudent_touch(modifier),
            Action::FocusedSynthesis => self.action_focused_synthesis(modifier),
            Action::FocusedTouch => self.action_focused_touch(modifier),
            Action::Reflect => self.action_reflect(modifier),
            Action::PreparatoryTouch => self.action_preparatory_touch(modifier),
            Action::Groundwork => self.action_groundwork(modifier),
            Action::DelicateSynthesis => self.action_delecate_synthesis(modifier),
            Action::IntensiveSynthesis => self.action_intensive_synthesis(modifier),
            Action::AdvancedTouch => self.action_advanced_touch(modifier),
            Action::HeartAndSoul => self.action_heart_and_soul(modifier),
            Action::PrudentSynthesis => self.action_prudent_synthesis(modifier),
            Action::TrainedFinesse => self.action_trained_finesse(modifier),
        }
    }
}

// Velvet Weissmelさんの調査式を元に計算
// https://jp.finalfantasyxiv.com/lodestone/character/3514261/blog/4645845/
pub fn get_technical_point(worth:u32) -> u32 {
    if worth < 5800 {
        0
    }
    else if worth < 6500 {
        (1*worth - 4050) / 10
    }
    else if worth < 7400 {
        (3*worth - 15800) / 10
    }
    else {
        (6*worth - 36200) / 10
    }
}

