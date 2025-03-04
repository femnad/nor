mod notify;
mod wpctl;

extern crate skim;

use crate::wpctl::volume::{apply, VolumeOp, OpType};
use clap::{Args, Command, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use std::io;

#[derive(Debug, Parser)]
#[command(name = "nor", version, about = "nor: Tiny wpctl wrapper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Default(DefaultArgs),
    #[command(about = "Generate completions")]
    Generate(GenerateArgs),
    #[command(about = "Show sinks and sources")]
    Status,
    Volume(VolumeArgs),
}

#[derive(Args, Debug)]
struct GenerateArgs {
    #[arg(help = "Shell name")]
    shell: Shell,
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
    #[command(about = "Show defaults")]
    Show(NodeArgs),
    #[command(about = "Set default sink")]
    Sink(NodeArgs),
    #[command(about = "Set default source")]
    Source(NodeArgs),
}

#[derive(Clone, Debug, ValueEnum)]
enum NodeType {
    Sink,
    Source,
}

#[derive(Args, Debug)]
struct NodeArgs {
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
    Dec {
        #[arg(default_value_t = NodeType::Sink, short = 't', long, value_enum)]
        node_type: NodeType,
        step: Option<u32> },
    #[command(about = "Get volume")]
    Get {
        #[arg(default_value_t = NodeType::Sink, short = 't', long, value_enum)]
        node_type: NodeType,
    },
    #[command(about = "Increase volume")]
    Inc {
        #[arg(default_value_t = NodeType::Sink, short = 't', long, value_enum)]
        node_type: NodeType,
        step: Option<u32>,
    },
    #[command(about = "Set volume")]
    Set {
        #[arg(default_value_t = NodeType::Sink, short = 't', long, value_enum)]
        node_type: NodeType,
        value: u32,
    },
    #[command(about = "Display volume")]
    Show {
        #[arg(default_value_t = NodeType::Sink, short = 't', long, value_enum)]
        node_type: NodeType,
    },
    #[command(about = "Toggle mute state")]
    Toggle {
        #[arg(default_value_t = NodeType::Sink, short = 't', long, value_enum)]
        node_type: NodeType,
    },
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Default(node) => match node.node {
            Node::Show(show) => wpctl::node::show_defaults(show.prefer_gui),
            Node::Reset => wpctl::node::reset_default(),
            Node::Sink(sink) => wpctl::node::set_default("sink", sink.prefer_gui),
            Node::Source(source) => wpctl::node::set_default("source", source.prefer_gui),
        },
        Commands::Generate(generate_args) => {
            let mut cmd = Cli::command();
            print_completions(generate_args.shell, &mut cmd);
        }
        Commands::Status => {
            wpctl::node::print_status();
        }
        Commands::Volume(op) => {
            let change = match op.op {
                Op::Dec { step, node_type } => VolumeOp::new(
                    OpType::Dec, step, node_type),
                Op::Get {node_type} => VolumeOp::new(
                    OpType::Get, None, node_type),
                Op::Inc { step, node_type } => VolumeOp::new(
                    OpType::Inc, step, node_type),
                Op::Set { value, node_type } => VolumeOp::new(
                    OpType::Set { value }, None, node_type),
                Op::Show { node_type } => VolumeOp::new(
                    OpType::Show, None, node_type),
                Op::Toggle {node_type } => VolumeOp::new(
                    OpType::Toggle, None, node_type),
            };
            apply(change);
        }
    }
}
