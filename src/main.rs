mod generate;
mod judge;
mod language;
mod solve;
mod utils;
mod validate;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "kuroe")]
#[command(about = "kuroe is a lightweight CLI tool for creating competitive programming problems", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    #[command(about = "generate testcases")]
    Generate(generate::GenerateArgs),

    #[command(arg_required_else_help = true)]
    #[command(about = "validate testcases")]
    Validate(validate::ValidateArgs),

    #[command(arg_required_else_help = true)]
    #[command(about = "generate solutions")]
    Solve(solve::SolveArgs),

    #[command(arg_required_else_help = true)]
    #[command(about = "judge a solver")]
    Judge(judge::JudgeArgs),
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Generate(args) => {
            generate::root(args).expect("failed to generate");
        }
        Commands::Validate(args) => {
            validate::root(args).expect("failed to validate");
        }
        Commands::Solve(args) => {
            println!("Solve {args:?}");
        }
        Commands::Judge(args) => {
            println!("Judge {args:?}");
        }
    }
}
