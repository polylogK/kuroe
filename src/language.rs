use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[derive(Debug)]
pub(crate) enum ExecuteStatus {
    Success,
    TimeLimitExceed,
    Fail,
}

impl ExecuteStatus {
    pub fn success(&self) -> bool {
        matches!(self, ExecuteStatus::Success)
    }
}

impl From<ExitStatus> for ExecuteStatus {
    fn from(status: ExitStatus) -> ExecuteStatus {
        if status.success() {
            ExecuteStatus::Success
        } else {
            ExecuteStatus::Fail
        }
    }
}

#[derive(Debug)]
pub(crate) struct CommandStep {
    program: String,
    args: Vec<String>,
    ignore_additional_args: bool,
}

impl CommandStep {
    pub(crate) fn new(program: String, args: Vec<String>) -> Self {
        Self {
            program,
            args,
            ignore_additional_args: false,
        }
    }

    pub(crate) fn new_ignore_additional_args(program: String, args: Vec<String>) -> Self {
        Self {
            program,
            args,
            ignore_additional_args: true,
        }
    }

    pub(crate) fn execute<P: AsRef<Path>, T: Into<Stdio>, U: Into<Stdio>, V: Into<Stdio>>(
        &self,
        current_dir: P,
        additional_args: Vec<String>,
        stdin: T,
        stdout: U,
        stderr: V,
        time_limit: Duration,
    ) -> Result<ExecuteStatus> {
        let args = if !self.ignore_additional_args {
            [&self.args[..], &additional_args[..]].concat()
        } else {
            self.args.clone()
        };

        let mut child = Command::new(&self.program)
            .args(args)
            .current_dir(current_dir)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .with_context(|| format!("Failed to execute {:?}", self))?;

        let status = match child.wait_timeout(time_limit)? {
            Some(status) => ExecuteStatus::from(status),
            None => {
                // child hasn't exited yet
                child.kill().unwrap();
                child.wait().unwrap();
                ExecuteStatus::TimeLimitExceed
            }
        };
        Ok(status)
    }
}

pub(crate) trait Language {
    fn is_valid_ext(&self, ext: &str) -> bool;
    fn compile(&self, target: &Path) -> Result<Vec<CommandStep>>;
    fn run(&self, target: &Path) -> Result<CommandStep>;
}

pub(crate) struct Clang;
impl Language for Clang {
    fn is_valid_ext(&self, ext: &str) -> bool {
        return ext == "c";
    }

    fn compile(&self, target: &Path) -> Result<Vec<CommandStep>> {
        Ok(vec![CommandStep::new(
            "gcc".to_string(),
            vec![
                "-std=c11".to_string(),
                "-O2".to_string(),
                target.canonicalize()?.to_string_lossy().to_string(),
            ],
        )])
    }

    fn run(&self, _target: &Path) -> Result<CommandStep> {
        Ok(CommandStep::new("./a.out".to_string(), Vec::new()))
    }
}

pub(crate) struct Cpp;
impl Language for Cpp {
    fn is_valid_ext(&self, ext: &str) -> bool {
        return ext == "cpp" || ext == "cc";
    }

    fn compile(&self, target: &Path) -> Result<Vec<CommandStep>> {
        Ok(vec![CommandStep::new(
            "g++".to_string(),
            vec![
                "-std=c++20".to_string(),
                "-O2".to_string(),
                target.canonicalize()?.to_string_lossy().to_string(),
            ],
        )])
    }

    fn run(&self, _target: &Path) -> Result<CommandStep> {
        Ok(CommandStep::new("./a.out".to_string(), Vec::new()))
    }
}

pub(crate) struct Python;
impl Language for Python {
    fn is_valid_ext(&self, ext: &str) -> bool {
        return ext == "py";
    }

    fn compile(&self, _target: &Path) -> Result<Vec<CommandStep>> {
        Ok(Vec::new())
    }

    fn run(&self, target: &Path) -> Result<CommandStep> {
        Ok(CommandStep::new(
            "python3".to_string(),
            vec![target.canonicalize()?.to_string_lossy().to_string()],
        ))
    }
}

pub(crate) struct Txt;
impl Language for Txt {
    fn is_valid_ext(&self, ext: &str) -> bool {
        return ext == "txt" || ext == "in";
    }

    fn compile(&self, _target: &Path) -> Result<Vec<CommandStep>> {
        Ok(Vec::new())
    }

    fn run(&self, target: &Path) -> Result<CommandStep> {
        Ok(CommandStep::new_ignore_additional_args(
            "cat".to_string(),
            vec![target.canonicalize()?.to_string_lossy().to_string()],
        ))
    }
}

pub(crate) struct CustomLang {
    ext: Regex,
    compile: Vec<String>,
    run: String,
}
impl CustomLang {
    pub(crate) fn new(ext: Regex, commands: Vec<String>) -> Result<Self> {
        let ext = format!("^({ext})$");
        let ext = Regex::new(&ext)?;
        let len = commands.len();
        ensure!(len >= 1, "commands.len() >= 1");

        Ok(Self {
            ext,
            compile: commands[0..(len - 1)].to_vec(),
            run: commands[len - 1].clone(),
        })
    }
}
impl Language for CustomLang {
    fn is_valid_ext(&self, ext: &str) -> bool {
        return self.ext.is_match(ext);
    }

    fn compile(&self, target: &Path) -> Result<Vec<CommandStep>> {
        let target = target.canonicalize()?.to_string_lossy().to_string();

        let mut cmds = Vec::new();
        for command in &self.compile {
            let command = command.replace("%(target)", &target);
            let parts: Vec<String> = command.split(' ').map(|s| s.to_string()).collect();

            cmds.push(CommandStep::new(parts[0].clone(), parts[1..].to_vec()));
        }
        Ok(cmds)
    }

    fn run(&self, target: &Path) -> Result<CommandStep> {
        let target = target.canonicalize()?.to_string_lossy().to_string();

        let command = self.run.replace("%(target)", &target);
        let parts: Vec<String> = command.split(' ').map(|s| s.to_string()).collect();

        Ok(CommandStep::new(parts[0].clone(), parts[1..].to_vec()))
    }
}

pub(crate) fn default_languages() -> Vec<Box<dyn Language + 'static>> {
    vec![
        Box::new(Clang),
        Box::new(Cpp),
        Box::new(Python),
        Box::new(Txt),
    ]
}

/// langs から ext に合った Language をクローンして返す
/// 複数の言語に合致する場合は先頭に近いものが優先される
/// カスタム言語で ext が被った場合はカスタム言語を先頭にすることで上書きすることが可能
pub(crate) fn detect_language<'a>(
    ext: &str,
    langs: &'a Vec<Box<dyn Language>>,
) -> Result<&'a Box<dyn Language>> {
    for lang in langs {
        if lang.is_valid_ext(ext) {
            return Ok(lang);
        }
    }
    bail!("no language detected");
}

/// target を compile して runstep を返す
pub(crate) fn compile_and_get_runstep<P: AsRef<Path>>(
    current_dir: P,
    target: &Path,
    langs: &Vec<Box<dyn Language>>,
) -> Result<CommandStep> {
    let lang = {
        let ext = target
            .extension()
            .with_context(|| format!("{:?} is not found", target))?
            .to_string_lossy()
            .to_string();
        detect_language(&ext, &langs)?
    };

    for step in lang.compile(&target)? {
        step.execute(
            &current_dir,
            Vec::new(),
            Stdio::null(),
            Stdio::null(),
            Stdio::null(),
            Duration::from_secs(10),
        )?;
    }

    lang.run(&target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{read_to_string, File};
    use tempfile::tempdir;

    #[test]
    fn test_execute_status() {
        assert!(ExecuteStatus::Success.success());
        assert!(!ExecuteStatus::TimeLimitExceed.success());
        assert!(!ExecuteStatus::Fail.success());
    }

    #[test]
    fn test_execute() {
        let step = CommandStep::new("true".to_string(), Vec::new());
        assert!(step
            .execute(
                "./",
                Vec::new(),
                Stdio::null(),
                Stdio::null(),
                Stdio::null(),
                Duration::from_secs(1)
            )
            .unwrap()
            .success());

        let step = CommandStep::new("false".to_string(), Vec::new());
        assert!(!step
            .execute(
                "./",
                Vec::new(),
                Stdio::null(),
                Stdio::null(),
                Stdio::null(),
                Duration::from_secs(1)
            )
            .unwrap()
            .success());
    }

    #[test]
    fn test_language() {
        assert!(Clang.is_valid_ext("c"));
        assert!(!Clang.is_valid_ext("test"));

        assert!(Cpp.is_valid_ext("cpp"));
        assert!(Cpp.is_valid_ext("cc"));
        assert!(!Cpp.is_valid_ext("test"));

        assert!(Python.is_valid_ext("py"));
        assert!(!Python.is_valid_ext("test"));

        assert!(Txt.is_valid_ext("txt"));
        assert!(Txt.is_valid_ext("in"));
        assert!(!Txt.is_valid_ext("test"));

        let cmd = Clang.run(Path::new("target")).unwrap();
        assert_eq!(cmd.program, "./a.out".to_string());
        assert_eq!(cmd.args.len(), 0);

        let cmd = Cpp.run(Path::new("target")).unwrap();
        assert_eq!(cmd.program, "./a.out".to_string());
        assert_eq!(cmd.args.len(), 0);

        let cmd = Python.run(Path::new("target")).unwrap();
        assert_eq!(cmd.program, "python3".to_string());
        assert_eq!(cmd.args.len(), 1);

        let cmd = Txt.run(Path::new("target")).unwrap();
        assert_eq!(cmd.program, "cat".to_string());
        assert_eq!(cmd.args.len(), 1);
    }

    #[test]
    fn test_custom_language() {
        let lang = CustomLang::new(Regex::new("rs").unwrap(), vec!["true".to_string()]).unwrap();
        assert!(lang.is_valid_ext("rs"));
        assert!(!lang.is_valid_ext("test"));
    }

    #[test]
    fn test_detect_language() {
        let langs: Vec<Box<dyn Language>> = vec![Box::new(Cpp), Box::new(Txt)];

        let lang = detect_language("cpp", &langs);
        assert!(lang.unwrap().is_valid_ext("cpp"));

        let lang = detect_language("txt", &langs);
        assert!(lang.unwrap().is_valid_ext("txt"));

        let lang = detect_language("test", &langs);
        assert!(lang.is_err());
    }

    #[test]
    fn test_compile_and_get_runstep() {
        let langs: Vec<Box<dyn Language>> = vec![Box::new(Cpp), Box::new(Txt)];
        let temp_dir = tempdir().unwrap();
        let temp_file = temp_dir.path().join("test.txt");
        let _ = File::create(&temp_file).unwrap();

        let runstep = compile_and_get_runstep(Path::new("./"), &temp_file, &langs).unwrap();
        assert_eq!(runstep.program, "cat".to_string());
        assert_eq!(runstep.args.len(), 1);
    }

    #[test]
    fn test_compile_and_run_cpp() {
        let lang = Cpp;
        let dir = tempdir().unwrap();

        // hello プログラムの作成
        let hello_path = dir.path().join("hello.cpp");
        let hello = File::create(&hello_path).unwrap();
        CommandStep::new(
            "echo".to_string(),
            vec!["#include <cstdio>\nint main(){ printf(\"hello\"); }".to_string()],
        )
        .execute(
            &dir,
            Vec::new(),
            Stdio::null(),
            hello,
            Stdio::null(),
            Duration::from_secs(2),
        )
        .unwrap();

        // コンパイル
        for step in lang.compile(&hello_path).unwrap() {
            step.execute(
                &dir,
                Vec::new(),
                Stdio::null(),
                Stdio::null(),
                Stdio::null(),
                Duration::from_secs(2),
            )
            .unwrap();
        }

        // 実行
        let output_path = dir.path().join("output.txt");
        let output = File::create(&output_path).unwrap();
        lang.run(&hello_path)
            .unwrap()
            .execute(
                &dir,
                Vec::new(),
                Stdio::null(),
                output,
                Stdio::null(),
                Duration::from_secs(2),
            )
            .unwrap();

        assert_eq!(read_to_string(&output_path).unwrap(), "hello");
    }

    #[test]
    fn test_compile_and_run_python() {
        let lang = Python;
        let dir = tempdir().unwrap();

        // hello プログラムの作成
        let hello_path = dir.path().join("hello.py");
        let hello = File::create(&hello_path).unwrap();
        CommandStep::new("echo".to_string(), vec!["print('hello')".to_string()])
            .execute(
                &dir,
                Vec::new(),
                Stdio::null(),
                hello,
                Stdio::null(),
                Duration::from_secs(2),
            )
            .unwrap();

        // コンパイル
        for step in lang.compile(&hello_path).unwrap() {
            step.execute(
                &dir,
                Vec::new(),
                Stdio::null(),
                Stdio::null(),
                Stdio::null(),
                Duration::from_secs(2),
            )
            .unwrap();
        }

        // 実行
        let output_path = dir.path().join("output.txt");
        let output = File::create(&output_path).unwrap();
        lang.run(&hello_path)
            .unwrap()
            .execute(
                &dir,
                Vec::new(),
                Stdio::null(),
                output,
                Stdio::null(),
                Duration::from_secs(2),
            )
            .unwrap();

        assert_eq!(read_to_string(&output_path).unwrap(), "hello\n");
    }

    #[test]
    fn test_compile_and_run_custom_lang() {
        let lang = CustomLang::new(
            Regex::new("cpp").unwrap(),
            vec![
                "g++ %(target) -o test".to_string(), // compile
                "./test".to_string(),                // execute
            ],
        )
        .unwrap();
        let dir = tempdir().unwrap();

        // hello プログラムの作成
        let hello_path = dir.path().join("hello.cpp");
        let hello = File::create(&hello_path).unwrap();
        CommandStep::new(
            "echo".to_string(),
            vec!["#include <cstdio>\nint main(int argc, char *argv[]) { printf(\"hello %s\", argv[1]); }".to_string()],
        )
        .execute(&dir, Vec::new(), Stdio::null(), hello,Stdio::null(),  Duration::from_secs(2))
        .unwrap();

        // コンパイル
        for step in lang.compile(&hello_path).unwrap() {
            step.execute(
                &dir,
                Vec::new(),
                Stdio::null(),
                Stdio::null(),
                Stdio::null(),
                Duration::from_secs(2),
            )
            .unwrap();
        }

        // 実行
        let output_path = dir.path().join("output.txt");
        let output = File::create(&output_path).unwrap();
        lang.run(&hello_path)
            .unwrap()
            .execute(
                &dir,
                vec!["0".to_string()],
                Stdio::null(),
                output,
                Stdio::null(),
                Duration::from_secs(2),
            )
            .unwrap();

        assert_eq!(read_to_string(&output_path).unwrap(), "hello 0");
    }
}
