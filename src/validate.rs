use crate::language::{compile_and_get_runstep, CommandStep, ExecuteStatus};
use crate::utils::{find_files, make_languages};
use anyhow::{bail, ensure, Result};
use clap::Args;
use log::{info, warn};
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug, Args)]
pub(super) struct ValidateArgs {
    /// path to the validator
    #[arg(value_name = "VALIDATOR", required = true)]
    validator: PathBuf,

    /// directory containing the testcases or path to the testcase(*.in)
    #[arg(short, long, default_value = "./testcases/input")]
    testcases: Vec<PathBuf>,

    /// recursively search for testcases
    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    ///
    #[arg(short, long, default_value = "./testcases/validate")]
    outdir: PathBuf,

    /// do not save the error outputs
    #[arg(short, long, default_value_t = false)]
    quiet: bool,

    /// COMMAND[0:-1] are the compile commands. COMMAND[-1] is execute command
    #[arg(
        short,
        long,
        value_name = "<EXT>,<COMMAND>,...",
        required = false,
        value_delimiter = ','
    )]
    language: Vec<String>,
}

/// vaildate の結果とエラー出力先パスを返す
fn validate<P: AsRef<Path>>(
    current_dir: P,
    target: &Path,
    outdir: &Path,
    run: &CommandStep,
    quiet: bool,
) -> Result<(ExecuteStatus, Option<PathBuf>)> {
    let input = File::open(&target)?;
    let name = target.file_stem().unwrap().to_string_lossy().to_string();

    if quiet {
        if let Ok(status) = run.execute(
            current_dir,
            Vec::new(),
            input,
            Stdio::null(),
            Stdio::null(),
            Duration::from_secs(10),
        ) {
            Ok((status, None))
        } else {
            bail!("failed to run")
        }
    } else {
        let err_path = outdir.join(format!("{name}.val"));
        let err = File::create(&err_path)?;

        if let Ok(status) = run.execute(
            current_dir,
            Vec::new(),
            input,
            Stdio::null(),
            err,
            Duration::from_secs(10),
        ) {
            Ok((status, Some(err_path.into())))
        } else {
            bail!("failed to run")
        }
    }
}

pub(super) fn root(args: ValidateArgs) -> Result<()> {
    info!("{:#?}", args);
    ensure!(
        args.validator.exists(),
        "validator {:?} not found",
        args.validator
    );

    let testcases = {
        let mut testcases = Vec::new();
        for base in args.testcases {
            let sub_files = find_files(&base, args.recursive).unwrap();

            for target in sub_files {
                if let Some(ext) = target.extension() {
                    if ext == "in" {
                        testcases.push(target);
                    }
                }
            }
        }
        testcases
    };
    if testcases.len() == 0 {
        warn!("no testcases found");
        return Ok(());
    }
    info!("testcases = {testcases:#?}");

    let langs = make_languages(&args.language)?;

    if args.quiet && !args.outdir.exists() {
        create_dir_all(&args.outdir)?;
    }

    let dir = TempDir::new()?;
    let runstep = compile_and_get_runstep(&dir, &args.validator, &langs)?;
    for target in testcases {
        match validate(&dir, &target, &args.outdir, &runstep, args.quiet) {
            Ok((status, output)) => {
                if let Some(path) = output {
                    info!(
                        "target = {:?}: output = {:?}, status = {:?}",
                        target, path, status
                    );
                } else {
                    info!("target = {:?}: status = {:?}", target, status);
                }
            }
            Err(err) => {
                warn!("reason = {:?}", err);
            }
        }
    }

    Ok(())
}
