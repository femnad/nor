mod notify;
mod wpctl;

extern crate skim;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "znv", version, about = "znv: Tiny wpctl wrapper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Default(DefaultArgs),
    #[command(about = "Show sinks and sources")]
    Status,
    Volume(VolumeArgs),
}

#[derive(Args, Debug)]
#[command(about = "Modify/toggle volume")]
struct VolumeArgs {
    #[command(subcommand)]
    op: Op,
}

#[derive(Args, Debug)]
#[command(about = "Set defaults")]
struct DefaultArgs {
    #[command(subcommand)]
    node: Node,
}

#[derive(Debug, Subcommand)]
enum Node {
    #[command(about = "Reset defaults")]
    Reset,
    #[command(about = "Set default sink")]
    Sink(SetterArgs),
    #[command(about = "Set default source")]
    Source(SetterArgs),
}

#[derive(Args, Debug)]
struct SetterArgs {
    #[arg(
        short = 'g',
        long,
        help = "Prefer using GUI facilities for selection and messages, like rofi and desktop notifications"
    )]
    prefer_gui: bool,
}

#[derive(Debug, Subcommand)]
enum Op {
    #[command(about = "Decrease volume")]
    Dec { step: Option<u32> },
    #[command(about = "Increase volume")]
    Inc { step: Option<u32> },
    #[command(about = "Toggle mute state")]
    Toggle,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Default(node) => match node.node {
            Node::Reset => wpctl::node::reset_default(),
            Node::Sink(sink) => wpctl::node::set_default("sink", sink.prefer_gui),
            Node::Source(source) => wpctl::node::set_default("source", source.prefer_gui),
        },
        Commands::Status => {
            let status = wpctl::node::get_status();
            println!("{status}");
        }
        Commands::Volume(op) => {
            let volume = match op.op {
                Op::Dec { step } | Op::Inc { step } => {
                    let sign = match op.op {
                        Op::Dec { .. } => "-",
                        Op::Inc { .. } => "+",
                        _ => unreachable!("No other Op variant should be matched here"),
                    };
                    wpctl::volume::modify(step, sign)
                }
                Op::Toggle => wpctl::volume::toggle(),
            };
            notify::volume(volume);
        }
    }
}
