use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(super) struct SolveArgs {
    /// path to the solver
    #[arg(value_name = "SOLVER", required = true)]
    solver: PathBuf,

    /// directory containing the testcases or path to the testcase(*.in)
    #[arg(value_name = "TARGET", required = true)]
    target: Vec<PathBuf>,

    ///
    #[arg(short, long, default_value = "./testcases")]
    outdir: PathBuf,
}
