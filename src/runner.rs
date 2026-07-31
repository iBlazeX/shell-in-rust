use std::{
    fs,
    io::{self, Write},
};

use crate::execute::spawn_external;
use crate::jobs::{Job, JobStatus};
use crate::{
    builtin::{BuiltinResult, run_builtin},
    shell::Shell,
    tokenizer::ParsedCmd,
};

pub enum ShellAction {
    Continue,
    Exit,
}

pub fn run(parsed: &ParsedCmd, shell: &mut Shell) -> ShellAction {
    let mut file;
    let mut errfile;

    let out: &mut dyn Write = if let Some(path) = &parsed.stout {
        file = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)
            .unwrap();

        &mut file
    } else {
        &mut io::stdout()
    };

    let err: &mut dyn Write = if let Some(path) = &parsed.sterr {
        errfile = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)
            .unwrap();

        &mut errfile
    } else {
        &mut io::stderr()
    };

    match run_builtin(parsed, shell, out, err) {
        BuiltinResult::Continue => ShellAction::Continue,
        BuiltinResult::Exit => ShellAction::Exit,
        BuiltinResult::NotBuiltin => match spawn_external(parsed, None, None, None) {
            Ok(mut child) => {
                if parsed.bg {
                    println!("[{}] {}", shell.next_job_id, child.id());

                    shell.jobs.push(Job {
                        id: shell.next_job_id,
                        child,
                        token: format!("{} {}", parsed.cmd, parsed.args.join(" ")),
                        status: JobStatus::Running,
                    });

                    shell.next_job_id += 1;
                } else {
                    child.wait().unwrap();
                }

                ShellAction::Continue
            }

            Err(_) => {
                writeln!(err, "{}: not found", parsed.cmd).unwrap();
                ShellAction::Continue
            }
        },
    }
}
