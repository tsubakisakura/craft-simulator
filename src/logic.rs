
extern crate xorshift;

use serde::Serialize;
use std::cmp::min;
use std::hash::Hash;
use xorshift::{Rng,Xorshift128};
use num::traits::{FromPrimitive,ToPrimitive};

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Hash)]
pub enum Condition
{
    Standard,       // 通常
    HighQuality,    // 高品質
    HighProgress,   // 高進捗
    HighEfficiency, // 高能率
    HighSustain,    // 高持続
    Solid,          // 頑丈
}

pub const ACTION_NUM: usize = 32;

#[derive(Debug,Clone,Serialize,PartialEq)]
pub enum Action {
    BasicSynthesis,     // 作業
    BasicTouch,         // 加工
    MastersMend,        // マスターズメンド
    HastyTouch,         // ヘイスティタッチ
    RapidSynthesis,     // 突貫作業
    InnerQuiet,         // インナークワイエット
    Observe,            // 経過観察
    TricksOfTheTrade,   // 秘訣
    WasteNot,           // 倹約
    Veneration,         // ヴェネレーション
    StandardTouch,      // 中級加工
    GreatStrides,       // グレートストライド
    Innovation,         // イノベーション
    NameOfTheElements,  // アートオブエレメンタル
    BrandOfTheElements, // ブランドオブエレメンタル
    FinalAppraisal,     // 最終確認
    WasteNot2,          // 長期倹約
    ByregotsBlessing,   // ビエルゴの祝福
    PreciseTouch,       // 集中加工
    MuscleMemory,       // 確信
    CarefulObservation, // 設計変更
    CarefulSynthesis,   // 模範作業
    PatientTouch,       // 専心加工
    Manipulation,       // マニピュレーション
    PrudentTouch,       // 倹約加工
    FocusedSynthesis,   // 注視作業
    FocusedTouch,       // 注視加工
    Reflect,            // 真価
    PreparatoryTouch,   // 下地加工
    Groundwork,         // 下地作業
    DelicateSynthesis,  // 精密作業
    IntensiveSynthesis, // 集中作業
//  TrainedEye,         // 匠の早業(非対応)
}

#[derive(Debug,Clone,Serialize,PartialEq,Eq,Hash)]
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
    pub elements : u32,               // 残アートオブエレメンタル
    pub final_appraisal : u32,        // 残最終確認
    pub muscle_memory : u32,          // 残確信
    pub manipulation : u32,           // マニピュレーション
    pub elements_used : bool,         // アートオブエレメンタルを使用済みかどうか
    pub combo_touch : bool,           // 直前に加工したかどうか
    pub combo_observe : bool,         // 直前に経過観察したかどうか
    pub condition : Condition         // 状態
}

#[derive(Debug,Clone)]
pub struct Setting
{
    pub max_working : u32,                // 必要工数
    pub max_quality : u32,                // 品質上限
    pub max_durability : u32,             // 初期耐久
    pub work_accuracy : u32,              // 作業精度
    pub process_accuracy : u32,           // 加工精度
    pub required_process_accuracy : u32,  // 必要加工精度
    pub max_cp : u32,                     // 初期CP
}

pub struct Modifier
{
    pub setting : Setting,
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
            5 => Some(Action::InnerQuiet),         // インナークワイエット
            6 => Some(Action::Observe),            // 経過観察
            7 => Some(Action::TricksOfTheTrade),   // 秘訣
            8 => Some(Action::WasteNot),           // 倹約
            9 => Some(Action::Veneration),         // ヴェネレーション
            10 => Some(Action::StandardTouch),      // 中級加工
            11 => Some(Action::GreatStrides),       // グレートストライド
            12 => Some(Action::Innovation),         // イノベーション
            13 => Some(Action::NameOfTheElements),  // アートオブエレメンタル
            14 => Some(Action::BrandOfTheElements), // ブランドオブエレメンタル
            15 => Some(Action::FinalAppraisal),     // 最終確認
            16 => Some(Action::WasteNot2),          // 長期倹約
            17 => Some(Action::ByregotsBlessing),   // ビエルゴの祝福
            18 => Some(Action::PreciseTouch),       // 集中加工
            19 => Some(Action::MuscleMemory),       // 確信
            20 => Some(Action::CarefulObservation), // 設計変更
            21 => Some(Action::CarefulSynthesis),   // 模範作業
            22 => Some(Action::PatientTouch),       // 専心加工
            23 => Some(Action::Manipulation),       // マニピュレーション
            24 => Some(Action::PrudentTouch),       // 倹約加工
            25 => Some(Action::FocusedSynthesis),   // 注視作業
            26 => Some(Action::FocusedTouch),       // 注視加工
            27 => Some(Action::Reflect),            // 真価
            28 => Some(Action::PreparatoryTouch),   // 下地加工
            29 => Some(Action::Groundwork),         // 下地作業
            30 => Some(Action::DelicateSynthesis),  // 精密作業
            31 => Some(Action::IntensiveSynthesis), // 集中作業
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
            Action::InnerQuiet => 5,
            Action::Observe => 6,
            Action::TricksOfTheTrade => 7,
            Action::WasteNot => 8,
            Action::Veneration => 9,
            Action::StandardTouch => 10,
            Action::GreatStrides => 11,
            Action::Innovation => 12,
            Action::NameOfTheElements => 13,
            Action::BrandOfTheElements => 14,
            Action::FinalAppraisal => 15,
            Action::WasteNot2 => 16,
            Action::ByregotsBlessing => 17,
            Action::PreciseTouch => 18,
            Action::MuscleMemory => 19,
            Action::CarefulObservation => 20,
            Action::CarefulSynthesis => 21,
            Action::PatientTouch => 22,
            Action::Manipulation => 23,
            Action::PrudentTouch => 24,
            Action::FocusedSynthesis => 25,
            Action::FocusedTouch => 26,
            Action::Reflect => 27,
            Action::PreparatoryTouch => 28,
            Action::Groundwork => 29,
            Action::DelicateSynthesis => 30,
            Action::IntensiveSynthesis => 31,
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

impl Setting {
    pub fn initial_state(&self) -> State {
        State {
            turn:1,
            time:0,
            completed:false,
            working:0,
            quality:0,
            durability:self.max_durability,
            cp:self.max_cp,
            inner_quiet:0,
            careful_observation:3,
            waste_not:0,
            veneration:0,
            great_strides:0,
            innovation:0,
            elements:0,
            final_appraisal:0,
            muscle_memory:0,
            manipulation:0,
            elements_used:false,
            combo_touch:false,
            combo_observe:false,
            condition:Condition::Standard,
        }
    }
}

impl State {
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
            Action::InnerQuiet => 18,
            Action::Observe => 7,
            Action::TricksOfTheTrade => 0,
            Action::WasteNot => 56,
            Action::Veneration => 18,
            Action::StandardTouch => if self.combo_touch { 18 } else { 32 },
            Action::GreatStrides => 32,
            Action::Innovation => 18,
            Action::NameOfTheElements => 30,
            Action::BrandOfTheElements => 8,
            Action::FinalAppraisal => 1,
            Action::WasteNot2 => 98,
            Action::ByregotsBlessing => 24,
            Action::PreciseTouch => 18,
            Action::MuscleMemory => 6,
            Action::CarefulObservation => 0,
            Action::CarefulSynthesis => 7,
            Action::PatientTouch => 6,
            Action::Manipulation => 96,
            Action::PrudentTouch => 25,
            Action::FocusedSynthesis => 5,
            Action::FocusedTouch => 18,
            Action::Reflect => 24,
            Action::PreparatoryTouch => 40,
            Action::Groundwork => 18,
            Action::DelicateSynthesis => 32,
            Action::IntensiveSynthesis => 6,
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
            _ if v < 0.73 => Condition::HighEfficiency,
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
            durability: if self.manipulation == 0 || self.is_terminated() { self.durability } else { min(self.durability+5,modifier.setting.max_durability) },
            veneration: decrement_clip(self.veneration),
            waste_not: decrement_clip(self.waste_not),
            great_strides: decrement_clip(self.great_strides),
            innovation: decrement_clip(self.innovation),
            elements: decrement_clip(self.elements),
            final_appraisal: decrement_clip(self.final_appraisal),
            muscle_memory: decrement_clip(self.muscle_memory),
            manipulation: decrement_clip(self.manipulation),
            combo_touch: false,
            combo_observe: false,
            .. *self
        }
    }

    fn duration(&self,x:u32) -> u32 {
        if self.condition == Condition::HighSustain { x+2 } else { x }
    }

    // 作業に対する品質報酬
    // 情報が無いので、そのまま決め打ちで打ち込んでます
    fn working_reward(&self, setting:&Setting, efficiency : f64 ) -> u32 {
        // 情報が無いのでそのまま決め打ちの数値の対応です。それ以外に対応することになったらやる
        if setting.work_accuracy != 2769 {
            return 99999;
        }

        let q = 472.0;
        let cond_rate = if self.condition == Condition::HighProgress { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if self.veneration > 0 { 0.5 } else { 0.0 } + if self.muscle_memory > 0 { 1.0 } else { 0.0 };

        return ( q * cond_rate * efficiency * buff_rate ) as u32;
    }

    // アートオブエレメンタル時のブランドオブエレメンタルの効率計算
    fn elements_efficiency(&self, setting:&Setting) -> u32 {
        let t = self.working as f32 / setting.max_working as f32;
        (102.0 + (1.0-t) * 198.0) as u32
    }

    // 効率に対する品質報酬
    // こちらの記事が紹介しているcalculatorの内容を参考にしています。
    // https://jp.finalfantasyxiv.com/lodestone/character/29523439/blog/4641394/
    // 完全一致はしませんが、近似値として使えます。完全一致を求めるならば、データシートを作るほうが良いと思う
    fn quality_reward(&self, setting:&Setting, efficiency : f64) -> u32 {
        let inner_quiet : f64 = From::from(self.inner_quiet);
        let process_accuracy : f64 = From::from(setting.process_accuracy);
        let required_process_accuracy : f64 = From::from(setting.required_process_accuracy);

        let f = if self.inner_quiet == 0 { process_accuracy } else { process_accuracy + process_accuracy * ((inner_quiet-1.0) * 20.0 / 100.0) };
        let q1 = f*35.0/100.0 + 35.0;
        let q2 = q1 * (f + 10000.0) / (required_process_accuracy + 10000.0);
        let q3 = q2 * 60.0 / 100.0;
        let cond_rate = if self.condition == Condition::HighQuality { 1.5 } else { 1.0 };
        let buff_rate = 1.0 + if self.great_strides > 0 { 1.0 } else { 0.0 } + if self.innovation > 0 { 0.5 } else { 0.0 };

        return ( q3 * cond_rate * efficiency * buff_rate ) as u32;
    }

    fn add_working(&self, setting:&Setting, efficiency : f64) -> State {
        let w = self.working + self.working_reward(&setting,efficiency);

        if w >= setting.max_working {
            if self.final_appraisal > 0 {
                State { working:setting.max_working - 1, muscle_memory:0, final_appraisal:0, .. *self } // 最終確認バフを消して完了直前に設定
            }
            else {
                State { working:setting.max_working, muscle_memory:0, completed: true, .. *self } // 作業完了
            }
        }
        else {
            State { working:w, muscle_memory:0, .. *self } // 未完なので足すだけ
        }
    }

    fn add_working_elements(&self, setting:&Setting) -> State {
        if self.elements == 0 {
            self.add_working(&setting, 1.0)
        }
        else {
            self.add_working(&setting, self.elements_efficiency(&setting) as f64 / 100.0)
        }
    }

    fn add_quality_base(&self, setting:&Setting, efficiency:f64) -> State {
        State {
            quality: min(self.quality + self.quality_reward(&setting,efficiency), setting.max_quality),
            great_strides: 0,
            .. *self
        }
    }

    fn add_inner_quiet(&self, inner_quiet_stack:u32) -> State {
        State {
            inner_quiet: if self.inner_quiet == 0 { 0 } else { min(self.inner_quiet+inner_quiet_stack,11) },
            .. *self
        }
    }

    fn sub_inner_quiet(&self, inner_quiet_stack:u32) -> State {
        State {
            inner_quiet: self.inner_quiet - inner_quiet_stack,
            .. *self
        }
    }

    fn add_time(&self, x:u32) -> State {
        State { time: self.time + x, .. * self }
    }

    fn set_inner_quiet(&self, inner_quiet:u32) -> State {
        State { inner_quiet: inner_quiet, .. *self }
    }

    fn add_quality(&self, setting:&Setting, efficiency:f64, inner_quiet_stack:u32) -> State {
        self.add_quality_base(&setting,efficiency).add_inner_quiet(inner_quiet_stack)
    }

    fn add_quality_byregots(&self, setting:&Setting) -> State {
        self.add_quality_base(&setting,1.0 + (self.inner_quiet-1) as f64 * 0.2).set_inner_quiet(0)
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

    fn add_durability(&self, x:u32, setting:&Setting) -> State {
        State { durability: min(self.durability+x,setting.max_durability), ..*self }
    }

    fn add_cp(&self, x:u32, setting:&Setting) -> State {
        State { cp: min(self.cp+x,setting.max_cp), ..*self }
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

    fn set_elements(&self, dur:u32) -> State {
        State { elements: self.duration(dur), elements_used:true, .. *self }
    }

    fn set_final_appraisal(&self, dur:u32) -> State {
        State { final_appraisal: self.duration(dur), .. *self }
    }

    fn set_muscle_memory(&self, dur:u32) -> State {
        State { muscle_memory: self.duration(dur), .. *self }
    }

    fn set_combo_touch(&self) -> State {
        State { combo_touch: true, .. *self }
    }

    fn set_combo_observe(&self) -> State {
        State { combo_observe: true, .. *self }
    }

    fn clear_combo(&self) -> State {
        State { combo_observe: false, combo_touch: false, .. *self }
    }

    // 実行確認
    pub fn check_action(&self, a:&Action) -> bool {
        if self.cp >= self.get_required_cp(&a) {
            match a {
                Action::InnerQuiet => self.inner_quiet == 0,
                Action::TricksOfTheTrade => self.condition == Condition::HighQuality,
                Action::NameOfTheElements => self.elements_used == false, // アートオブエレメンタル使用済みだと使えません
                Action::ByregotsBlessing => self.inner_quiet > 1, // ビエルゴはinner_quiet初期値の時は使えません
                Action::PreciseTouch => self.condition == Condition::HighQuality,
                Action::MuscleMemory => self.turn == 1, // 確信バフは最終確認で消えません
                Action::CarefulObservation => self.careful_observation > 0,
                Action::PatientTouch => self.inner_quiet != 0,
                Action::PrudentTouch => self.waste_not == 0,
                Action::Reflect => self.turn == 1, // 真価バフは最終確認で消えません
                Action::IntensiveSynthesis => self.condition == Condition::HighQuality,
                _ => true
            }
        }
        else {
            false
        }
    }

    // 作業
    fn action_basic_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.setting,1.2).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 加工
    fn action_basic_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.setting,1.0,1).consume_cp(&Action::BasicTouch).consume_durability(10).next_turn(modifier).set_combo_touch().change_condition(modifier).add_time(3)
    }

    // マスターズメンド
    fn action_masters_mend(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::MastersMend).add_durability(30,&modifier.setting).next_turn(modifier).change_condition(modifier).add_time(2)
    }

    // ヘイスティタッチ
    fn action_hasty_touch(&self, modifier:&mut Modifier) -> State {
        if modifier.try_random(0.5) {
            // 成功時
            self.add_quality(&modifier.setting,1.0,1).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗時
            self.consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // 突貫作業
    fn action_rapid_synthesis(&self, modifier:&mut Modifier) -> State {
        if modifier.try_random(0.5) {
            // 成功時
            self.add_working(&modifier.setting,5.0).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗時
            self.consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // インナークワイエット
    fn action_inner_quiet(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::HastyTouch).next_turn(modifier).set_inner_quiet(1).change_condition(modifier).add_time(2)
    }

    // 経過観察
    fn action_observe(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::Observe).next_turn(modifier).set_combo_observe().change_condition(modifier).add_time(3)
    }

    // 秘訣
    fn action_trick_of_the_trade(&self, modifier:&mut Modifier) -> State {
        self.next_turn(modifier).add_cp(20,&modifier.setting).change_condition(modifier).add_time(3)
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
        self.add_quality(&modifier.setting,1.25,1).consume_cp(&Action::StandardTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // グレートストライド
    fn action_great_strides(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::GreatStrides).next_turn(modifier).set_great_strides(3).change_condition(modifier).add_time(2)
    }

    // イノベーション
    fn action_innovation(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::Innovation).next_turn(modifier).set_innovation(4).change_condition(modifier).add_time(2)
    }

    // アートオブエレメンタル
    fn action_name_of_elements(&self, modifier:&mut Modifier) -> State {
        self.consume_cp(&Action::NameOfTheElements).next_turn(modifier).set_elements(3).change_condition(modifier).add_time(2)
    }

    // ブランドオブエレメンタル
    fn action_brand_of_elements(&self, modifier:&mut Modifier) -> State {
        self.add_working_elements(&modifier.setting).consume_cp(&Action::BrandOfTheElements).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
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
        self.add_quality_byregots(&modifier.setting).consume_cp(&Action::ByregotsBlessing).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 集中加工
    fn action_precise_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.setting,1.5,2).consume_cp(&Action::PreciseTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 確信
    fn action_muscle_memory(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.setting,3.0).consume_cp(&Action::MuscleMemory).consume_durability(10).next_turn(modifier).set_muscle_memory(5).change_condition(modifier).add_time(3)
    }

    // 設計変更
    fn action_careful_observation(&self, modifier:&mut Modifier) -> State {
        self.clear_combo().consume_careful_observation().change_condition(modifier).add_time(2)
    }

    // 模範作業
    fn action_careful_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.setting,1.6).consume_cp(&Action::CarefulSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 専心加工
    fn action_patient_touch(&self, modifier:&mut Modifier) -> State {
        if modifier.try_random(0.5) {
            // 成功の場合
            self.add_quality_base(&modifier.setting,1.0).add_inner_quiet(self.inner_quiet).consume_cp(&Action::PatientTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗の場合
            self.sub_inner_quiet(self.inner_quiet/2).consume_cp(&Action::PatientTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // マニピュレーション
    fn action_manipulation(&self, modifier:&mut Modifier) -> State {
        self.clear_manipulation().consume_cp(&Action::Manipulation).next_turn(modifier).set_manipulation(8).change_condition(modifier).add_time(2)
    }

    // 倹約加工
    fn action_prudent_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.setting,1.0,1).consume_cp(&Action::PrudentTouch).consume_durability(5).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 注視作業
    fn action_focused_synthesis(&self, modifier:&mut Modifier) -> State {
        if self.combo_observe || modifier.try_random(0.5) {
            // 成功の場合
            self.add_working(&modifier.setting,1.5).consume_cp(&Action::FocusedSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗の場合
            self.consume_cp(&Action::FocusedSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // 注視作業
    fn action_focused_touch(&self, modifier:&mut Modifier) -> State {
        if self.combo_observe || modifier.try_random(0.5) {
            // 成功の場合
            self.add_quality(&modifier.setting,1.5,1).consume_cp(&Action::FocusedTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
        else {
            // 失敗の場合
            self.consume_cp(&Action::FocusedTouch).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
        }
    }

    // 真価
    fn action_reflect(&self, modifier:&mut Modifier) -> State {
        self.add_quality_base(&modifier.setting,1.0).set_inner_quiet(3).consume_cp(&Action::Reflect).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 下地加工
    fn action_preparatory_touch(&self, modifier:&mut Modifier) -> State {
        self.add_quality(&modifier.setting,2.0,2).consume_cp(&Action::PreparatoryTouch).consume_durability(20).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 下地作業
    fn action_groundwork(&self, modifier:&mut Modifier) -> State {
        let efficiency = if self.get_required_cp(&Action::Groundwork) < self.durability as u32 { 1.5 } else { 3.0 };

        self.add_working(&modifier.setting,efficiency).consume_cp(&Action::Groundwork).consume_durability(20).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 精密作業
    fn action_delecate_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.setting,1.0).add_quality(&modifier.setting,1.0,1).consume_cp(&Action::DelicateSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // 集中作業
    fn action_intensive_synthesis(&self, modifier:&mut Modifier) -> State {
        self.add_working(&modifier.setting,4.0).consume_cp(&Action::IntensiveSynthesis).consume_durability(10).next_turn(modifier).change_condition(modifier).add_time(3)
    }

    // アクション取得
    pub fn run_action(&self, modifier:&mut Modifier, a:&Action) -> State {
        match a {
        Action::BasicSynthesis => self.action_basic_synthesis(modifier),
            Action::BasicTouch => self.action_basic_touch(modifier),
            Action::MastersMend => self.action_masters_mend(modifier),
            Action::HastyTouch => self.action_hasty_touch(modifier),
            Action::RapidSynthesis => self.action_rapid_synthesis(modifier),
            Action::InnerQuiet => self.action_inner_quiet(modifier),
            Action::Observe => self.action_observe(modifier),
            Action::TricksOfTheTrade => self.action_trick_of_the_trade(modifier),
            Action::WasteNot => self.action_waste_not(modifier),
            Action::Veneration => self.action_veneration(modifier),
            Action::StandardTouch => self.action_standard_touch(modifier),
            Action::GreatStrides => self.action_great_strides(modifier),
            Action::Innovation => self.action_innovation(modifier),
            Action::NameOfTheElements => self.action_name_of_elements(modifier),
            Action::BrandOfTheElements => self.action_brand_of_elements(modifier),
            Action::FinalAppraisal => self.action_final_apprisal(modifier),
            Action::WasteNot2 => self.action_waste_not2(modifier),
            Action::ByregotsBlessing => self.action_byregots_blessing(modifier),
            Action::PreciseTouch => self.action_precise_touch(modifier),
            Action::MuscleMemory => self.action_muscle_memory(modifier),
            Action::CarefulObservation => self.action_careful_observation(modifier),
            Action::CarefulSynthesis => self.action_careful_synthesis(modifier),
            Action::PatientTouch => self.action_patient_touch(modifier),
            Action::Manipulation => self.action_manipulation(modifier),
            Action::PrudentTouch => self.action_prudent_touch(modifier),
            Action::FocusedSynthesis => self.action_focused_synthesis(modifier),
            Action::FocusedTouch => self.action_focused_touch(modifier),
            Action::Reflect => self.action_reflect(modifier),
            Action::PreparatoryTouch => self.action_preparatory_touch(modifier),
            Action::Groundwork => self.action_groundwork(modifier),
            Action::DelicateSynthesis => self.action_delecate_synthesis(modifier),
            Action::IntensiveSynthesis => self.action_intensive_synthesis(modifier),
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

#[allow(dead_code)]
fn test_initial_setting() -> Setting {
    Setting {
        max_working:12046,
        max_quality:81447,
        max_durability:55,
        work_accuracy:2769,
        process_accuracy:2840,
        required_process_accuracy:2540,
        max_cp:569,
    }
}

// テストのみで使うので許す
#[allow(dead_code)]
fn diff<T : std::cmp::PartialOrd + std::ops::Sub<Output=T>>( left:T, right:T ) -> T {
    if left <= right { right - left } else { left - right }
}

#[test]
fn test_working()
{
    let setting = test_initial_setting();
    let s0 = setting.initial_state();
    let sv = State { veneration:1, .. s0.clone() };
    let sp = State { veneration:1, condition:Condition::HighProgress, .. s0.clone() };

    // 実測値です(作業制度2769でff14にて確認)
    assert_eq!( 472, s0.working_reward(&setting,1.0) );
    assert_eq!( 566, s0.working_reward(&setting,1.2) );
    assert_eq!( 708, s0.working_reward(&setting,1.5) );
    assert_eq!( 944, s0.working_reward(&setting,2.0) );
    assert_eq!( 1416, s0.working_reward(&setting,3.0) );
    assert_eq!( 2360, s0.working_reward(&setting,5.0) );

    assert_eq!( 708, sv.working_reward(&setting,1.0) );
    assert_eq!( 849, sv.working_reward(&setting,1.2) );
    assert_eq!( 1062, sv.working_reward(&setting,1.5) );
    assert_eq!( 1416, sv.working_reward(&setting,2.0) );
    assert_eq!( 2124, sv.working_reward(&setting,3.0) );
    assert_eq!( 3540, sv.working_reward(&setting,5.0) );

    assert_eq!( 1062, sp.working_reward(&setting,1.0) );
    assert_eq!( 1274, sp.working_reward(&setting,1.2) );
    assert_eq!( 1593, sp.working_reward(&setting,1.5) );
    assert_eq!( 2124, sp.working_reward(&setting,2.0) );
    assert_eq!( 3186, sp.working_reward(&setting,3.0) );
    assert_eq!( 5310, sp.working_reward(&setting,5.0) );
}

#[allow(dead_code)]
fn calc_quality( inner_quiet:u32 ) -> u32
{
    let setting = test_initial_setting();
    let state = State { inner_quiet:inner_quiet, .. setting.initial_state() };

    state.quality_reward(&setting,1.0)
}

#[test]
fn test_quality()
{
    assert!( diff( calc_quality(1), 634 ) < 3 );
    assert!( diff( calc_quality(2), 787 ) < 3 );
    assert!( diff( calc_quality(3), 953 ) < 3 );
    assert!( diff( calc_quality(4), 1131 ) < 3 );
    assert!( diff( calc_quality(5), 1319 ) < 3 );
    assert!( diff( calc_quality(6), 1517 ) < 3 );
    assert!( diff( calc_quality(7), 1727 ) < 3 );
    assert!( diff( calc_quality(8), 1947 ) < 3 );
    assert!( diff( calc_quality(9), 2178 ) < 3 );
    assert!( diff( calc_quality(10), 2420 ) < 3 );
    assert!( diff( calc_quality(11), 2673 ) < 3 );
}

// アートオブエレメンタルの実測値のテスト用
#[allow(dead_code)]
fn calc_elements_efficiency( working:u32 ) -> u32
{
    let setting = test_initial_setting();
    let state = State { working:working, .. setting.initial_state() };

    state.elements_efficiency(&setting)
}

#[test]
fn test_elements()
{
    // 実測値を元にした比較
    // 割と誤差あるようです
    assert!( diff( calc_elements_efficiency(0), 300 ) < 3 );
    assert!( diff( calc_elements_efficiency(472), 294 ) < 3 );
    assert!( diff( calc_elements_efficiency(566), 292 ) < 3 );
    assert!( diff( calc_elements_efficiency(708), 290 ) < 3 );
    assert!( diff( calc_elements_efficiency(944), 286 ) < 3 );
    assert!( diff( calc_elements_efficiency(1416), 278 ) < 3 );
    assert!( diff( calc_elements_efficiency(1859), 270 ) < 3 );
    assert!( diff( calc_elements_efficiency(1944), 267 ) < 3 );
    assert!( diff( calc_elements_efficiency(2076), 266 ) < 3 );
    assert!( diff( calc_elements_efficiency(2293), 262 ) < 3 );
    assert!( diff( calc_elements_efficiency(2360), 262 ) < 3 );
    assert!( diff( calc_elements_efficiency(2728), 256 ) < 3 );
    assert!( diff( calc_elements_efficiency(3133), 248 ) < 3 );
    assert!( diff( calc_elements_efficiency(3208), 248 ) < 3 );
    assert!( diff( calc_elements_efficiency(3331), 246 ) < 3 );
    assert!( diff( calc_elements_efficiency(3529), 242 ) < 3 );
    assert!( diff( calc_elements_efficiency(3540), 242 ) < 3 );
    assert!( diff( calc_elements_efficiency(3596), 242 ) < 3 );
    assert!( diff( calc_elements_efficiency(4682), 224 ) < 3 );
    assert!( diff( calc_elements_efficiency(4738), 222 ) < 3 );
    assert!( diff( calc_elements_efficiency(5739), 206 ) < 3 );
    assert!( diff( calc_elements_efficiency(7080), 183 ) < 3 );
    assert!( diff( calc_elements_efficiency(7948), 170 ) < 3 );
    assert!( diff( calc_elements_efficiency(8673), 158 ) < 3 );
    assert!( diff( calc_elements_efficiency(8750), 156 ) < 3 );
    assert!( diff( calc_elements_efficiency(9418), 144 ) < 3 );
    assert!( diff( calc_elements_efficiency(10437), 128 ) < 3 );
    assert!( diff( calc_elements_efficiency(10620), 124 ) < 3 );
    assert!( diff( calc_elements_efficiency(11205), 114 ) < 3 );
    assert!( diff( calc_elements_efficiency(11743), 105 ) < 3 );
    assert!( diff( calc_elements_efficiency(12045), 102 ) < 3 );
}

#[allow(dead_code)]
fn calc_byregots_result( inner_quiet_stack:u32 ) -> u32 {
    let setting = test_initial_setting();
    let state = State { inner_quiet:inner_quiet_stack, .. setting.initial_state() };

    state.add_quality_byregots(&setting).quality
}

#[test]
fn test_byregots()
{
    // 実測値を元にした比較
    assert!( diff( calc_byregots_result(2), 944 ) < 3 );
    assert!( diff( calc_byregots_result(3), 1334 ) < 3 );
    assert!( diff( calc_byregots_result(4), 1809 ) < 3 );
    assert!( diff( calc_byregots_result(5), 2374 ) < 3 );
    assert!( diff( calc_byregots_result(6), 3034 ) < 3 );
    assert!( diff( calc_byregots_result(7), 3799 ) < 3 );
    assert!( diff( calc_byregots_result(8), 4672 ) < 3 );
    assert!( diff( calc_byregots_result(9), 5662 ) < 3 );
    assert!( diff( calc_byregots_result(10), 6776 ) < 3 );
    assert!( diff( calc_byregots_result(11), 8019 ) < 3 );
}

#[test]
fn test_technical_point()
{
    assert_eq!( 175, get_technical_point(5800));
    assert_eq!( 244, get_technical_point(6499));
    assert_eq!( 370, get_technical_point(6500));
    assert_eq!( 639, get_technical_point(7399));
    assert_eq!( 820, get_technical_point(7400));
    assert_eq!(1266, get_technical_point(8144));
}

#[test]
fn test_action_conv_primitive()
{
    for i in 0..ACTION_NUM {
        assert_eq!( i as u64, Action::from_u64(i as u64).unwrap().to_u64().unwrap() );
    }
}
