use crate::language::{compile_and_get_runstep, CommandStep, ExecuteStatus};
use crate::utils::{find_files, make_languages};
use anyhow::{bail, ensure, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug, Args)]
pub(super) struct JudgeArgs {
    /// path to the solver
    #[arg(value_name = "SOLVER", required = true)]
    solver: PathBuf,

    /// path to the checker
    #[arg(short, long)]
    checker: Option<PathBuf>,

    /// directory containing the testcases(*.in and *.ans)
    #[arg(short, long, value_name = "TESTCASE", default_value = "./testcases")]
    testcase: PathBuf,

    ///
    #[arg(short, long, default_value = "./testcases/output")]
    outdir: PathBuf,

    /// timelimit for solver
    #[arg(visible_alias = "tl", long, default_value_t = 2.0)]
    timelimit: f64,

    /// judge policy
    // todo!()

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

#[derive(Debug, Clone)]
struct JudgeInfo {
    input_path: Option<PathBuf>,
    answer_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    status: Option<ExecuteStatus>,
}

impl JudgeInfo {
    fn new() -> Self {
        Self {
            input_path: None,
            answer_path: None,
            output_path: None,
            status: None,
        }
    }

    fn input(mut self, path: &Path) -> Self {
        self.input_path = Some(path.to_path_buf());
        self
    }
    fn answer(mut self, path: &Path) -> Self {
        self.answer_path = Some(path.to_path_buf());
        self
    }
    fn output(mut self, path: &Path) -> Self {
        self.output_path = Some(path.to_path_buf());
        self
    }
    fn status(mut self, status: ExecuteStatus) -> Self {
        self.status = Some(status);
        self
    }

    fn get_input_path(&self) -> Option<&PathBuf> {
        self.input_path.as_ref()
    }
    fn get_answer_path(&self) -> Option<&PathBuf> {
        self.answer_path.as_ref()
    }
    fn get_output_path(&self) -> Option<&PathBuf> {
        self.output_path.as_ref()
    }
}

/// .in と .ans が揃っているケースを列挙
fn enumerate_valid_testcases(all_cases: &Vec<PathBuf>) -> Vec<JudgeInfo> {
    let mut ans_cases = HashMap::new();
    for case in all_cases.iter() {
        if case.extension().map_or(false, |ext| ext == "ans") {
            ans_cases.insert(case.file_stem().unwrap(), case);
        }
    }

    let mut valid_cases = Vec::new();
    for case in all_cases {
        if case.extension().map_or(false, |ext| ext == "in") {
            let base_name = case.file_stem().unwrap();

            if let Some(ans_path) = ans_cases.get(base_name) {
                valid_cases.push(JudgeInfo::new().input(&case).answer(&ans_path));
            }
        }
    }

    valid_cases.sort_by(|x, y| x.get_input_path().cmp(&y.get_input_path()));
    valid_cases
}

/// output 出力先を返す
fn solve<P: AsRef<Path>>(
    current_dir: P,
    target: &Path,
    outdir: &Path,
    run: &CommandStep,
    timelimit: f64,
) -> Result<(ExecuteStatus, PathBuf)> {
    let input = File::open(&target)?;

    let name = target.file_stem().unwrap().to_string_lossy().to_string();
    let output_path = outdir.join(format!("{name}.out"));
    let output = File::create(&output_path)?;

    if let Ok(status) = run.execute(
        current_dir,
        Vec::new(),
        input,
        output,
        Stdio::null(),
        Duration::from_secs_f64(timelimit),
    ) {
        Ok((status, output_path))
    } else {
        bail!("failed to run")
    }
}

/// 完全一致ジャッジ
fn judge_by_diff<P: AsRef<Path>>(current_dir: P, info: &JudgeInfo) -> Result<bool> {
    let answer = info
        .get_answer_path()
        .unwrap()
        .canonicalize()?
        .to_string_lossy()
        .to_string();
    let output = info
        .get_output_path()
        .unwrap()
        .canonicalize()?
        .to_string_lossy()
        .to_string();

    Ok(CommandStep::new(format!("diff"), Vec::new())
        .execute(
            current_dir,
            vec![answer, output],
            Stdio::null(),
            Stdio::null(),
            Stdio::null(),
            Duration::from_secs(10),
        )?
        .success())
}

// checker によるジャッジ
fn judge<P: AsRef<Path>>(current_dir: P, info: &JudgeInfo, run: &CommandStep) -> Result<bool> {
    let input = info
        .get_input_path()
        .unwrap()
        .canonicalize()?
        .to_string_lossy()
        .to_string();
    let answer = info
        .get_answer_path()
        .unwrap()
        .canonicalize()?
        .to_string_lossy()
        .to_string();
    let output = info
        .get_output_path()
        .unwrap()
        .canonicalize()?
        .to_string_lossy()
        .to_string();

    if let Ok(status) = run.execute(
        current_dir,
        vec![input, output, answer],
        Stdio::null(),
        Stdio::null(),
        Stdio::null(),
        Duration::from_secs(10),
    ) {
        Ok(status.success())
    } else {
        bail!("failed to run")
    }
}

pub(super) fn root(args: JudgeArgs) -> Result<()> {
    info!("{:#?}", args);
    ensure!(args.solver.exists(), "solver {:?} not found", args.solver);

    let langs = make_languages(&args.language)?;

    if !args.outdir.exists() {
        create_dir_all(&args.outdir)?;
    }

    let mut testcases = {
        let all_cases = find_files(&args.testcase, true)?;
        enumerate_valid_testcases(&all_cases)
    };
    if testcases.len() == 0 {
        warn!("no testcases found");
        return Ok(());
    }

    // generate outputs
    let dir = TempDir::new()?;
    let runstep = compile_and_get_runstep(&dir, &args.solver, &langs)?;
    let bar = ProgressBar::new(testcases.len() as u64);
    bar.set_style(ProgressStyle::default_bar().template("[Solve] {bar} {pos:>4}/{len:4}")?);
    for target in testcases.iter_mut() {
        match solve(
            &dir,
            target.get_input_path().unwrap(),
            &args.outdir,
            &runstep,
            args.timelimit,
        ) {
            Ok((status, output)) => {
                info!("[OUTPUT] {:?}, status = {:?}", output, status);

                *target = target.clone().output(&output).status(status);
            }
            Err(err) => {
                warn!("[IGNORED] {:?}, reason {:?}", target, err);
            }
        }
        bar.inc(1);
    }
    bar.finish();

    // judge
    if let Some(checker) = args.checker {
        ensure!(checker.exists(), "checker {checker:?} not found");

        let dir = TempDir::new()?;
        let runstep = compile_and_get_runstep(&dir, &checker, &langs)?;
        let bar = ProgressBar::new(testcases.len() as u64);
        bar.set_style(ProgressStyle::default_bar().template("[Judge] {bar} {pos:>4}/{len:4}")?);
        for target in testcases.iter() {
            match judge(&dir, target, &runstep) {
                Ok(status) => {
                    info!("[JUDGE] {:#?}, status = {:?}", target, status);
                }
                Err(err) => {
                    warn!("[JUDGE FAILED] {:?}, reason = {:?}", target, err);
                }
            }
            bar.inc(1);
        }
        bar.finish();
    } else {
        let bar = ProgressBar::new(testcases.len() as u64);
        bar.set_style(ProgressStyle::default_bar().template("[Judge] {bar} {pos:>4}/{len:4}")?);
        for target in testcases.iter() {
            match judge_by_diff(&dir, target) {
                Ok(status) => {
                    info!("[JUDGE] {:#?}, status = {:?}", target, status);
                }
                Err(err) => {
                    warn!("[JUDGE FAILED] {:?}, reason = {:?}", target, err);
                }
            }
            bar.inc(1);
        }
        bar.finish();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_valid_testcases() {
        let cases = enumerate_valid_testcases(&vec![
            PathBuf::from("input/test.in"),
            PathBuf::from("answer/test.ans"),
        ]);
        assert_eq!(cases.len(), 1);
        assert_eq!(
            cases[0].get_input_path(),
            Some(&PathBuf::from("input/test.in"))
        );
        assert_eq!(
            cases[0].get_answer_path(),
            Some(&PathBuf::from("answer/test.ans"))
        );

        let cases = enumerate_valid_testcases(&vec![
            PathBuf::from("input/test.in"),
            PathBuf::from("answer/invalid.ans"),
        ]);
        assert_eq!(cases.len(), 0);
    }

    #[test]
    fn test_judge_file_info() {
        let input_path = PathBuf::from("test.in");
        let answer_path = PathBuf::from("test.ans");
        let output_path = PathBuf::from("test.out");

        let info = JudgeInfo::new()
            .input(&input_path)
            .answer(&answer_path)
            .output(&output_path)
            .status(ExecuteStatus::TimeLimitExceed);
        assert_eq!(info.get_input_path(), Some(&input_path));
        assert_eq!(info.get_answer_path(), Some(&answer_path));
        assert_eq!(info.get_output_path(), Some(&output_path));
        assert_eq!(info.status, Some(ExecuteStatus::TimeLimitExceed));

        let info = JudgeInfo::new().input(&input_path).answer(&answer_path);
        assert_eq!(info.get_input_path(), Some(&input_path));
        assert_eq!(info.get_answer_path(), Some(&answer_path));
        assert_eq!(info.get_output_path(), None);
        assert_eq!(info.status, None);
    }
}
