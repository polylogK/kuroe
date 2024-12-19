use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(super) struct JudgeArgs {
    /// path to the solver
    #[arg(value_name = "SOLVER", required = true)]
    solver: PathBuf,

    /// directory containing the testcases(*.in)
    #[arg(value_name = "TESTCASE", default_value = "./testcases")]
    indir: PathBuf,

    /// directory containing the testcases(*.out)
    #[arg(value_name = "SOLUTION", default_value = "./testcases")]
    soldir: PathBuf,

    /// saving dicrectory for solver output
    #[arg(short, long)]
    outdir: Option<PathBuf>,
}
