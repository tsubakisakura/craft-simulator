// クラフトシミュレータ by Tsubaki Sakura

mod logic;
mod cui;
mod selfplay;
mod mcts;
mod formatter;
mod network;
mod network2;
mod writer;
mod selector;
mod gcs;
mod cache;
mod learner;
mod benchmark;
mod executor;
mod predictor;
mod replay;
mod setting;

use setting::{Setting,ModifierParameter};
use argh::FromArgs;
use selfplay::{WriterParameter,EpisodeParameter,SelfPlayParameter};
use selector::Selector;
use learner::{LearnerParameter};
use benchmark::BenchmarkParameter;
use network2::NetworkType;
use cui::{CuiParameter};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description="toplevel command")]
struct TopLevel {
    #[argh(subcommand)]
    sub_command: SubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    Evaluator(SubCommandEvaluator),
    Generator(SubCommandGenerator),
    Learner(SubCommandLearner),
    Benchmark(SubCommandBenchmark),
    Replay(SubCommandReplay),
    Cui(SubCommandCui),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name="evaluator", description="generate evaluator")]
struct SubCommandEvaluator {
    #[argh(option, default="10", description="plays per write")]
    plays_per_write:usize,

    #[argh(option, default="4", description="thread num")]
    thread_num:u32,

    #[argh(option, default="16", description="batch size")]
    batch_size:usize,

    #[argh(option, default="500", description="mcts simulation num")]
    mcts_simulation_num:u32,

    #[argh(option, description="use ucb1 selector")]
    ucb1:Option<f64>,

    #[argh(option, description="use optimistic selector")]
    optimistic:Option<usize>,

    #[argh(option, description="use greedy selector")]
    greedy:Option<usize>,

    #[argh(option, default="1", description="torch parallelism thread num")]
    tch_thread_num:u32,

    #[argh(option, default="1", description="torch interop thread num")]
    tch_interop_thread_num:u32,

    #[argh(option, default="String::from(\"root\")", description="mysql user name")]
    mysql_user:String,

    #[argh(switch, description="profile with flamegraph")]
    flamegraph: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name="generator", description="generate samples")]
struct SubCommandGenerator {
    #[argh(option, default="100", description="plays per write")]
    plays_per_write:usize,

    #[argh(option, default="4", description="thread num")]
    thread_num:u32,

    #[argh(option, default="32", description="batch size")]
    batch_size:usize,

    #[argh(option, default="500", description="mcts simulation num")]
    mcts_simulation_num:u32,

    #[argh(option, default="0.15", description="dirichlet noise alpha")]
    alpha:f32,

    #[argh(option, default="0.3", description="dirichlet noise epsilon(0 for no noise)")]
    eps:f32,

    #[argh(option, default="30", description="start greety algorithm turn")]
    start_greedy_turn:u32,

    #[argh(option, description="use ucb1 selector")]
    ucb1:Option<f64>,

    #[argh(option, description="use optimistic selector")]
    optimistic:Option<usize>,

    #[argh(option, description="use greedy selector")]
    greedy:Option<usize>,

    #[argh(option, default="1", description="torch parallelism thread num")]
    tch_thread_num:u32,

    #[argh(option, default="1", description="torch interop thread num")]
    tch_interop_thread_num:u32,

    #[argh(option, default="String::from(\"root\")", description="mysql user name")]
    mysql_user:String,

    #[argh(switch, description="profile with flamegraph")]
    flamegraph: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name="learner", description="learning network")]
struct SubCommandLearner {
    #[argh(option, default="300", description="epochs per write")]
    epochs_per_write:usize,

    #[argh(option, default="40000", description="record buffer size")]
    record_buffer_size:usize,

    #[argh(option, default="String::from(\"root\")", description="mysql user name")]
    mysql_user:String,

    #[argh(option, default="NetworkType::FullyConnected(4,128)", description="network type")]
    network_type: NetworkType,

    #[argh(switch, description="profile with flamegraph")]
    flamegraph: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name="benchmark", description="benchmark network prediction")]
struct SubCommandBenchmark {
    #[argh(option, default="32", description="batch size")]
    batch_size:usize,

    #[argh(option, default="16384", description="plays per write")]
    plays_per_write:usize,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name="replay", description="replay record")]
struct SubCommandReplay {
    #[argh(positional, description="record name")]
    record_names: Vec<String>
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name="cui", description="CUI for crafting")]
struct SubCommandCui {
}

fn initial_setting() -> Setting {
    Setting {
        max_working:12046,
        max_quality:81447,
        max_durability:55,
        work_accuracy:2769,
        //process_accuracy:2840,
        process_accuracy:2840 + 70,
        required_process_accuracy:2540,
        //max_cp:569,
        max_cp:569 + 72 + 16,
    }
}

fn get_selector( ucb1:Option<f64>, optimistic:Option<usize>, greedy:Option<usize> ) -> Option<Selector> {
    if let Some(x) = ucb1 {
        Some(Selector::UCB1(x))
    }
    else if let Some(x) = optimistic {
        Some(Selector::Optimistic(x))
    }
    else if let Some(x) = greedy {
        Some(Selector::Greedy(x))
    }
    else {
        None
    }
}

fn with_flamegraph<F: FnOnce()>( f:F ) {
    let guard = pprof::ProfilerGuard::new(100).unwrap();
    f();
    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    }
}

fn cmd_evaluator( args:SubCommandEvaluator ) {
    let param = SelfPlayParameter {
        episode_param: EpisodeParameter {
            mod_param:ModifierParameter::new(&initial_setting()),
            mcts_simulation_num:args.mcts_simulation_num,
            alpha:0.15,
            eps:0.0,
            start_greedy_turn:0,
        },
        selector:get_selector(args.ucb1, args.optimistic, args.greedy).unwrap_or(Selector::Optimistic(10)),
        plays_per_write:args.plays_per_write,
        thread_num:args.thread_num,
        batch_size:args.batch_size,
        tch_thread_num:args.tch_thread_num,
        tch_interop_thread_num:args.tch_interop_thread_num,
        mysql_user:args.mysql_user,
        writer_param:WriterParameter::Evaluation,
    };

    if args.flamegraph {
        with_flamegraph( ||{ selfplay::run(&param) } );
    }
    else {
        selfplay::run(&param);
    }
}

fn cmd_generator( args:SubCommandGenerator ) {
    let param = SelfPlayParameter {
        episode_param: EpisodeParameter {
            mod_param:ModifierParameter::new(&initial_setting()),
            mcts_simulation_num:args.mcts_simulation_num,
            alpha:args.alpha,
            eps:args.eps,
            start_greedy_turn:args.start_greedy_turn,
        },
        selector:get_selector(args.ucb1, args.optimistic, args.greedy).unwrap_or(Selector::Greedy(50)),
        plays_per_write:args.plays_per_write,
        thread_num:args.thread_num,
        batch_size:args.batch_size,
        tch_thread_num:args.tch_thread_num,
        tch_interop_thread_num:args.tch_interop_thread_num,
        mysql_user:args.mysql_user,
        writer_param:WriterParameter::Generation,
    };

    if args.flamegraph {
        with_flamegraph( ||{ selfplay::run(&param) } );
    }
    else {
        selfplay::run(&param);
    }
}

fn cmd_learner( args:SubCommandLearner ) {
    let param = LearnerParameter {
        epochs_per_write:args.epochs_per_write,
        network_type:args.network_type,
        record_buffer_size:args.record_buffer_size,
        mysql_user:args.mysql_user,
    };

    if args.flamegraph {
        with_flamegraph( ||{ learner::run(&param) } );
    }
    else {
        learner::run(&param);
    }
}

fn cmd_benchmark( args:SubCommandBenchmark ) {
    let param = BenchmarkParameter {
        mod_param:ModifierParameter::new(&initial_setting()),
        batch_size:args.batch_size,
        plays_per_write:args.plays_per_write,
    };

    benchmark::run_benchmark(param);
}

fn cmd_replay( args: SubCommandReplay ) {
    replay::run_replay( args.record_names );
}

fn cmd_cui( _args:SubCommandCui ) {
    let param = CuiParameter {
        mod_param:ModifierParameter::new(&initial_setting()),
    };

    cui::run_cui(param);
}

fn main() {
    let cmdline: TopLevel = argh::from_env();

    match cmdline.sub_command {
        SubCommand::Evaluator(x) => cmd_evaluator(x),
        SubCommand::Generator(x) => cmd_generator(x),
        SubCommand::Learner(x) => cmd_learner(x),
        SubCommand::Benchmark(x) => cmd_benchmark(x),
        SubCommand::Replay(x) => cmd_replay(x),
        SubCommand::Cui(x) => cmd_cui(x),
    }
}
