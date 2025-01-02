mod generate;
mod judge;
mod language;
mod solve;
mod utils;
mod validate;

use clap::{Parser, Subcommand};
use env_logger;
use log::error;
use std::process::ExitCode;

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

fn main() -> ExitCode {
    env_logger::init();

    let args = Cli::parse();
    match args.command {
        Commands::Generate(args) => {
            if let Err(err) = generate::root(args) {
                error!("{err:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Commands::Validate(args) => {
            if let Err(err) = validate::root(args) {
                error!("{err:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Commands::Solve(args) => {
            if let Err(err) = solve::root(args) {
                error!("{err:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Commands::Judge(args) => {
            if let Err(err) = judge::root(args) {
                error!("{err:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}
