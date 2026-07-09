mod jobs;
mod runner;
mod tokenizer;
use jobs::Job;
use jobs::reap;
#[allow(unused_imports)]
use runner::{ShellAction, run};
use std::{fs, io, io::Write};
use tokenizer::tokenize;

pub struct Shell {
    pub jobs: Vec<Job>,
    pub next_job_id: usize,
    pub history: Vec<String>,
}

fn main() {
    let mut shell = Shell {
        jobs: Vec::new(),
        next_job_id: 1,
        history: Vec::new(),
    };
    let i: usize;
    loop {
        reap(&mut shell);
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.trim().is_empty() {
            continue;
        }
        shell
            .history
            .push(format!("{} {}", i, command.trim().to_string()));
        let parsed = tokenize(command.trim());
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
        if shell.jobs.is_empty() {
            shell.next_job_id = 1;
        } else {
            shell.next_job_id = shell.jobs.last().map(|job| job.id).unwrap() + 1;
        }
        match run(&parsed, &mut shell, out, err) {
            ShellAction::Exit => break,
            ShellAction::Continue => {}
        }
    }
}
