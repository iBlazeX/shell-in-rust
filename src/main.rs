mod jobs;
mod runner;
mod tokenizer;
use jobs::Job;
use jobs::reap;
mod expand;
#[allow(unused_imports)]
use runner::{ShellAction, run};
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::{fs, io, io::Write};
use tokenizer::tokenize;

pub struct Shell {
    pub jobs: Vec<Job>,
    pub next_job_id: usize,
    pub history: Vec<String>,
    pub vars: HashMap<String, String>,
}

fn main() {
    let mut shell = Shell {
        jobs: Vec::new(),
        next_job_id: 1,
        history: Vec::new(),
        vars: HashMap::new(),
    };
    let mut i: usize = 0;
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let command = match rl.readline("$ ") {
            Ok(line) => line,
            Err(_) => break,
        };
        if command.trim().is_empty() {
            continue;
        } else {
            rl.add_history_entry(command.as_str()).unwrap();
        }
        i += 1;
        shell
            .history
            .push(format!("{} {}", i, command.trim().to_string()));
        let mut parsed = tokenize(command.trim());
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
        expand::expand_command(&mut parsed, &shell);
        match run(&parsed, &mut shell, out, err) {
            ShellAction::Exit => break,
            ShellAction::Continue => {}
        }
        reap(&mut shell);
    }
}
