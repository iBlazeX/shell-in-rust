mod expand;
mod jobs;
mod runner;
mod tokenizer;

use expand::expand_command;
use jobs::{Job, reap};
use runner::{ShellAction, run};
use rustyline::DefaultEditor;
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
};
use tokenizer::{ParsedCmd, tokenize};

pub struct Shell {
    pub jobs: Vec<Job>,
    pub next_job_id: usize,
    pub history: Vec<String>,
    pub vars: HashMap<String, String>,
}

fn parse_commands(line: &str) -> Vec<ParsedCmd> {
    line.split('|').map(|cmd| tokenize(cmd.trim())).collect()
}

fn update_history(shell: &mut Shell, rl: &mut DefaultEditor, command: &str, counter: &mut usize) {
    rl.add_history_entry(command).unwrap();

    *counter += 1;
    shell
        .history
        .push(format!("{} {}", *counter, command.trim()));
}

fn update_next_job_id(shell: &mut Shell) {
    shell.next_job_id = shell.jobs.last().map(|job| job.id + 1).unwrap_or(1);
}

fn main() {
    let mut shell = Shell {
        jobs: Vec::new(),
        next_job_id: 1,
        history: Vec::new(),
        vars: HashMap::new(),
    };

    let mut history_count = 0;
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let command = match rl.readline("$ ") {
            Ok(line) => line,
            Err(_) => break,
        };

        if command.trim().is_empty() {
            continue;
        }

        update_history(&mut shell, &mut rl, &command, &mut history_count);

        let mut commands = parse_commands(&command);

        for parsed in &mut commands {
            expand_command(parsed, &shell);
        }

        update_next_job_id(&mut shell);

        if commands.len() == 1 {
            let parsed = &mut commands[0];

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

            match run(parsed, &mut shell, out, err) {
                ShellAction::Exit => break,
                ShellAction::Continue => {}
            }
        } else {
            // run_pipeline(&commands, &mut shell);
        }

        reap(&mut shell);
    }
}
