use crate::language::{compile_and_get_runstep, CommandStep, ExecuteStatus, Language};
use crate::utils::{find_files, make_languages};
use anyhow::{bail, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tabled::{Table, Tabled};
use tempfile::TempDir;

#[derive(Debug, Args)]
pub(super) struct ValidateArgs {
    /// path to the validator
    #[arg(value_name = "VALIDATOR", required = true)]
    validators: Vec<PathBuf>,

    /// recursively search for validator
    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    /// directory containing the testcases or path to the testcase(*.in)
    #[arg(short, long, default_value = "./testcases/input")]
    testcases: Vec<PathBuf>,

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

fn validate_root(
    validator: &Path,
    testcases: &Vec<PathBuf>,
    langs: &Vec<Box<dyn Language>>,
    outdir: &Path,
    quiet: bool,
) -> Result<()> {
    let dir = TempDir::new()?;
    let runstep = compile_and_get_runstep(&dir, validator, langs)?;

    let outdir = outdir.join(validator.file_stem().unwrap().to_str().unwrap());
    if !quiet && !outdir.exists() {
        create_dir_all(&outdir)?;
    }

    let bar = ProgressBar::new(testcases.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!("[{validator:?}] {{bar}} {{pos:>4}}/{{len:4}}"))?,
    );
    if quiet {
        #[derive(Tabled)]
        struct Result {
            status: String,
            target: String,
        }
        let mut results = Vec::new();

        for target in testcases {
            match validate(&dir, target, &outdir, &runstep, quiet) {
                Ok((status, None)) => {
                    info!("[VALIDATE] target = {:?}: status = {:?}", target, status);

                    results.push(Result {
                        status: status.to_string(),
                        target: format!("{:?}", target),
                    });
                }
                Err(err) => {
                    warn!("[VALIDATE] reason = {:?}", err);
                }
                _ => {
                    unreachable!();
                }
            }
            bar.inc(1);
        }
        bar.finish();

        println!("{}", Table::new(results));
    } else {
        #[derive(Tabled)]
        struct Result {
            status: String,
            target: String,
            stderr: String,
        }
        let mut results = Vec::new();

        for target in testcases {
            match validate(&dir, target, &outdir, &runstep, quiet) {
                Ok((status, Some(path))) => {
                    info!(
                        "[VALIDATE] target = {:?}: output = {:?}, status = {:?}",
                        target, path, status
                    );

                    results.push(Result {
                        status: status.to_string(),
                        target: format!("{:?}", target),
                        stderr: format!("{:?}", path),
                    });
                }
                Err(err) => {
                    warn!("[VALIDATE] reason = {:?}", err);
                }
                _ => {
                    unreachable!();
                }
            }
            bar.inc(1);
        }
        bar.finish();

        println!("{}", Table::new(results));
    }

    Ok(())
}

pub(super) fn root(args: ValidateArgs) -> Result<()> {
    info!("{:#?}", args);

    let validators = {
        let mut validators = Vec::new();
        for base in args.validators {
            for file in find_files(&base, args.recursive)? {
                validators.push(file);
            }
        }
        validators
    };
    if validators.len() == 0 {
        println!("no validator found!");
        return Ok(());
    }
    info!("validators = {validators:#?}");

    let testcases = {
        let mut testcases = Vec::new();
        for base in args.testcases {
            let sub_files = find_files(&base, false)?;

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
        println!("no testcase found!");
        return Ok(());
    }
    info!("testcases = {testcases:#?}");

    let langs = make_languages(&args.language)?;

    for (i, validator) in validators.iter().enumerate() {
        validate_root(&validator, &testcases, &langs, &args.outdir, args.quiet)?;

        if i + 1 < validators.len() {
            println!("");
        }
    }

    Ok(())
}
