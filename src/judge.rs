use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(super) struct JudgeArgs {
    /// path to the solver
    #[arg(value_name = "SOLVER", required = true)]
    solver: PathBuf,

    /// directory containing the testcases(*.in and *.ans)
    #[arg(value_name = "TESTCASE", default_value = "./testcases")]
    testcases: PathBuf,

    /// saving dicrectory for output
    #[arg(short, long, default_value = "./testcases/output")]
    outdir: PathBuf,
}
