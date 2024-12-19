use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub(super) struct ValidateArgs {
    /// path to the validator
    #[arg(value_name = "VALIDATOR", required = true)]
    validator: PathBuf,

    /// directory containing the testcases or path to the testcase(*.in)
    #[arg(value_name = "TARGET", required = true)]
    target: Vec<PathBuf>,
}
