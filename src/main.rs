mod jobs;
mod runner;
mod tokenizer;
use jobs::Job;
#[allow(unused_imports)]
use runner::{ShellAction, run};
use std::{fs, io, io::Write};
use tokenizer::tokenize;

pub struct Shell {
    pub jobs: Vec<Job>,
    pub next_job_id: usize,
}

fn main() {
    let mut shell = Shell {
        jobs: Vec::new(),
        next_job_id: 1,
    };
    loop {
        reap(&mut shell);
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.trim().is_empty() {
            continue;
        }
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
        match run(&parsed, &mut shell, out, err) {
            ShellAction::Exit => break,
            ShellAction::Continue => {}
        }
    }
}

fn reap(shell: &mut Shell) {
    shell
        .jobs
        .retain_mut(|job| match job.child.try_wait().unwrap() {
            Some(_) => {
                println!("[{}]+  Done                 {}", job.id, job.token);
                false
            }
            None => true,
        });
}
