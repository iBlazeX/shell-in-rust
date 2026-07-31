use std::{
    collections::HashMap,
    env,
    fs::{self, Metadata},
    io::{self, Write},
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::builtin::{BuiltinResult, is_builtin, run_builtin};

use crate::{
    jobs::{Job, JobStatus},
    runner::ShellAction,
    shell::Shell,
    tokenizer::ParsedCmd,
};

fn is_exec(meta: &Metadata) -> bool {
    meta.permissions().mode() & 0o111 != 0
}

pub fn find_exec(cmd: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH").unwrap();

    for dir in env::split_paths(&path) {
        let candidate = dir.join(cmd);

        if !candidate.is_file() {
            continue;
        }

        let meta = fs::metadata(&candidate).unwrap();

        if is_exec(&meta) {
            return Some(candidate);
        }
    }

    None
}

fn build_command(parsed: &ParsedCmd) -> Option<Command> {
    let path = find_exec(&parsed.cmd)?;

    let mut command = Command::new(path);
    command.arg0(&parsed.cmd);
    command.args(&parsed.args);

    Some(command)
}

pub fn run_external(parsed: &ParsedCmd, shell: &mut Shell, err: &mut dyn Write) -> ShellAction {
    let mut command = match build_command(parsed) {
        Some(cmd) => cmd,
        None => {
            writeln!(err, "{}: not found", parsed.cmd).unwrap();
            return ShellAction::Continue;
        }
    };

    if let Some(path) = &parsed.stout {
        let file = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)
            .unwrap();

        command.stdout(file);
    }

    if let Some(path) = &parsed.sterr {
        let file = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)
            .unwrap();

        command.stderr(file);
    }

    if parsed.bg {
        let child = command.spawn().unwrap();

        println!("[{}] {}", shell.next_job_id, child.id());

        shell.jobs.push(Job {
            id: shell.next_job_id,
            child,
            token: format!("{} {}", parsed.cmd, parsed.args.join(" ")),
            status: JobStatus::Running,
        });

        shell.next_job_id += 1;
    } else {
        command.status().unwrap();
    }

    ShellAction::Continue
}

pub fn run_pipeline(commands: &[ParsedCmd]) {
    let mut err = io::stderr();

    if commands.len() != 2 {
        writeln!(err, "Only two-command pipelines are supported").unwrap();
        return;
    }

    let left = &commands[0];
    let right = &commands[1];
    let left_builtin = is_builtin(&left.cmd);
    let right_builtin = is_builtin(&right.cmd);

    if left_builtin && !right_builtin {
        let mut buffer = Vec::new();

        let mut shell = Shell {
            jobs: Vec::new(),
            next_job_id: 1,
            history: Vec::new(),
            vars: HashMap::new(),
        };

        let mut stderr = io::stderr();

        match run_builtin(left, &mut shell, &mut buffer, &mut stderr) {
            BuiltinResult::Exit | BuiltinResult::Continue => {}
            BuiltinResult::NotBuiltin => unreachable!(),
        }

        let mut command = match build_command(right) {
            Some(cmd) => cmd,
            None => {
                writeln!(stderr, "{}: not found", right.cmd).unwrap();
                return;
            }
        };

        command.stdin(Stdio::piped());

        let mut child = command.spawn().unwrap();

        child.stdin.as_mut().unwrap().write_all(&buffer).unwrap();

        drop(child.stdin.take());

        child.wait().unwrap();

        return;
    }

    if !left_builtin && right_builtin {
        let mut shell = Shell {
            jobs: Vec::new(),
            next_job_id: 1,
            history: Vec::new(),
            vars: HashMap::new(),
        };

        let mut stderr = io::stderr();

        let mut command = match build_command(left) {
            Some(cmd) => cmd,
            None => {
                writeln!(stderr, "{}: not found", left.cmd).unwrap();
                return;
            }
        };

        command.stdout(Stdio::piped());

        let mut child = command.spawn().unwrap();

        // We intentionally ignore the pipe output.
        drop(child.stdout.take());

        let mut out = io::stdout();

        match run_builtin(right, &mut shell, &mut out, &mut stderr) {
            BuiltinResult::Exit | BuiltinResult::Continue => {}
            BuiltinResult::NotBuiltin => unreachable!(),
        }

        child.wait().unwrap();

        return;
    }

    let mut left_cmd = match build_command(left) {
        Some(cmd) => cmd,
        None => {
            writeln!(err, "{}: not found", left.cmd).unwrap();
            return;
        }
    };

    let mut right_cmd = match build_command(right) {
        Some(cmd) => cmd,
        None => {
            writeln!(err, "{}: not found", right.cmd).unwrap();
            return;
        }
    };

    left_cmd.stdout(Stdio::piped());

    let mut left_child = left_cmd.spawn().unwrap();

    let stdout = left_child.stdout.take().unwrap();

    right_cmd.stdin(Stdio::from(stdout));

    let mut right_child = right_cmd.spawn().unwrap();

    left_child.wait().unwrap();
    right_child.wait().unwrap();
}
