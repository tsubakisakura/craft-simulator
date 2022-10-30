
use std::time::SystemTime;
use super::logic::{Action,Modifier,State,Condition,get_technical_point};
use super::setting::ModifierParameter;
use xorshift::{SeedableRng};

pub struct CuiParameter {
    pub mod_param : ModifierParameter
}

fn parse_action( cmd:&str ) -> Option<Action> {
    match cmd {
        "a" => Some(Action::BasicSynthesis),
        "b" => Some(Action::BasicTouch),
        "c" => Some(Action::MastersMend),
        "d" => Some(Action::HastyTouch),
        "e" => Some(Action::RapidSynthesis),
        "g" => Some(Action::Observe),
        "h" => Some(Action::TricksOfTheTrade),
        "i" => Some(Action::WasteNot),
        "j" => Some(Action::Veneration),
        "k" => Some(Action::StandardTouch),
        "l" => Some(Action::GreatStrides),
        "m" => Some(Action::Innovation),
        "p" => Some(Action::FinalAppraisal),
        "q" => Some(Action::WasteNot2),
        "r" => Some(Action::ByregotsBlessing),
        "s" => Some(Action::PreciseTouch),
        "t" => Some(Action::MuscleMemory),
        "u" => Some(Action::CarefulObservation),
        "v" => Some(Action::CarefulSynthesis),
        "x" => Some(Action::Manipulation),
        "y" => Some(Action::PrudentTouch),
        "z" => Some(Action::FocusedSynthesis),
        "A" => Some(Action::FocusedTouch),
        "B" => Some(Action::Reflect),
        "C" => Some(Action::PreparatoryTouch),
        "D" => Some(Action::Groundwork),
        "E" => Some(Action::DelicateSynthesis),
        "F" => Some(Action::IntensiveSynthesis),
        "G" => Some(Action::AdvancedTouch),
        "H" => Some(Action::HeartAndSoul),
        "I" => Some(Action::PrudentSynthesis),
        "J" => Some(Action::TrainedFinesse),
        _ => None,
    }
}

impl Action {
    pub fn translate_ja(&self) -> &'static str {
        match *self {
            Action::BasicSynthesis => "作業",
            Action::BasicTouch => "加工",
            Action::MastersMend => "マスターズメンド",
            Action::HastyTouch => "ヘイスティタッチ",
            Action::RapidSynthesis => "突貫作業",
            Action::Observe => "経過観察",
            Action::TricksOfTheTrade => "秘訣",
            Action::WasteNot => "倹約",
            Action::Veneration => "ヴェネレーション",
            Action::StandardTouch => "中級加工",
            Action::GreatStrides => "グレートストライド",
            Action::Innovation => "イノベーション",
            Action::FinalAppraisal => "最終確認",
            Action::WasteNot2 => "長期倹約",
            Action::ByregotsBlessing => "ビエルゴの祝福",
            Action::PreciseTouch => "集中加工",
            Action::MuscleMemory => "確信",
            Action::CarefulObservation => "設計変更",
            Action::CarefulSynthesis => "模範作業",
            Action::Manipulation => "マニピュレーション",
            Action::PrudentTouch => "倹約加工",
            Action::FocusedSynthesis => "注視作業",
            Action::FocusedTouch => "注視加工",
            Action::Reflect => "真価",
            Action::PreparatoryTouch => "下地加工",
            Action::Groundwork => "下地作業",
            Action::DelicateSynthesis => "精密作業",
            Action::IntensiveSynthesis => "集中作業",
            Action::AdvancedTouch => "上級加工",
            Action::HeartAndSoul => "一心不乱",
            Action::PrudentSynthesis => "倹約加工",
            Action::TrainedFinesse => "匠の神業",
        }
    }
}

impl Condition {
    pub fn translate_ja(&self) -> &'static str {
        match *self {
            Condition::Standard => "通常",
            Condition::HighQuality => "高品質",
            Condition::HighProgress => "高進捗",
            Condition::HighEfficiency => "高能率",
            Condition::HighSustain => "高持続",
            Condition::Solid => "頑丈",
            Condition::Stable => "安定",
        }
    }

    pub fn get_color_escape(&self) -> &'static str {
        match *self {
            Condition::Standard => "\x1b[38;2;255;255;255m",
            Condition::HighQuality => "\x1b[38;2;255;128;128m",
            Condition::HighProgress => "\x1b[38;2;64;64;255m",
            Condition::HighEfficiency => "\x1b[38;2;128;255;128m",
            Condition::HighSustain => "\x1b[38;2;255;255;128m",
            Condition::Solid => "\x1b[38;2;128;128;255m",
            Condition::Stable => "\x1b[38;2;255;255;64m"
        }
    }
}

fn print_action() {
    let lowers = (0..26).map(|x| (x + b'a') as char);
    let uppers = (0..26).map(|x| (x + b'A') as char);
    let alphabets = lowers.into_iter().chain( uppers.into_iter() );

    println!("=====================");
    for c in alphabets {
        if let Some(action) = parse_action( &c.to_string() ) {
            println!("{}:{}({:?})", c, action.translate_ja(), action )
        }
    }
    println!("=====================");
}

fn get_normal_color_escape() -> &'static str {
    "\x1b[39m"
}

fn print_state(s:&State,mod_param:&ModifierParameter) {
    print!("\x1b[2J"); // 画面クリア
    print!("\x1b[0;0H"); // 左上移動
    print_action();

    println!("TURN:{}", s.turn);
    println!("作業:{}/{}", s.working, mod_param.max_working);
    println!("品質:{}/{}", s.quality, mod_param.max_quality);
    println!("耐久:{}/{}", s.durability, mod_param.max_durability);
    println!("ＣＰ:{}/{}", s.cp, mod_param.max_cp);
    println!("状態:{}●{}{}", s.condition.get_color_escape(), get_normal_color_escape(), s.condition.translate_ja() );
    println!("=====================");

    if s.inner_quiet > 0 {
        println!("インナークワイエット:{}",s.inner_quiet);
    }

    if s.careful_observation > 0 {
        println!("残設計変更:{}",s.careful_observation);
    }

    if s.waste_not > 0 {
        println!("残倹約:{}",s.waste_not);
    }

    if s.veneration > 0 {
        println!("残ヴェネレーション:{}",s.veneration);
    }

    if s.great_strides > 0 {
        println!("残グレートストライド:{}",s.great_strides);
    }

    if s.innovation > 0 {
        println!("残イノベーション:{}",s.innovation);
    }

    if s.final_appraisal > 0 {
        println!("残最終確認:{}",s.final_appraisal);
    }

    if s.muscle_memory > 0 {
        println!("残確信:{}",s.muscle_memory);
    }

    if s.manipulation > 0 {
        println!("残マニピュレーション:{}",s.manipulation);
    }

    if s.heart_and_soul {
        println!("一心不乱有効");
    }

    if s.heart_and_soul_used {
        println!("一心不乱再使用不可");
    }

    if s.combo_observe {
        println!("注視作業/注視加工100%");
    }

    if s.combo_basic_touch {
        println!("中級加工CP低下");
    }

    if s.combo_standard_touch {
        println!("上級加工CP低下");
    }
}

pub fn run_cui( param:CuiParameter ) {
    let seed : u64 = From::from( SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Failed to get UNIXTIME").subsec_nanos() );
    let states = [seed, seed];
    let mut modifier = Modifier { mod_param:param.mod_param.clone(), rng:SeedableRng::from_seed(&states[..]) };
    let mut state = State::new(&param.mod_param);

    while !state.is_terminated() {
        print_state(&state, &param.mod_param);

        let mut cmd = String::new();
        std::io::stdin().read_line(&mut cmd).expect("Failed to read_line");

        if let Some(action) = parse_action(cmd.trim()) {
            if state.check_action(&action) {
                state = state.run_action(&mut modifier,&action)
            }
            else {
                println!("Don't satisfy condition of [{:?}]", action);
            }
        }
        else {
            print_action();
        }
    }

    print_state(&state, &param.mod_param);
    if state.is_destroyed() {
        println!("Destroyed => Technical Point:0" );
    }
    else {
        println!("Completed => Technical Point:{}", get_technical_point(state.quality/10) );
    }
}
