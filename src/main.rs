mod builtin;
mod execute;
mod expand;
mod jobs;
mod pipeline;
mod runner;
mod shell;
mod tokenizer;

use crate::pipeline::run_pipeline;
use expand::expand_command;
use jobs::reap;
use runner::{ShellAction, run};
use rustyline::DefaultEditor;
use shell::Shell;
use std::collections::HashMap;
use tokenizer::{ParsedCmd, tokenize};

fn parse_commands(line: &str) -> Vec<ParsedCmd> {
    line.split('|').map(|s| tokenize(s.trim())).collect()
}

fn update_history(shell: &mut Shell, rl: &mut DefaultEditor, command: &str, counter: &mut usize) {
    rl.add_history_entry(command).unwrap();

    *counter += 1;
    shell
        .history
        .push(format!("{} {}", *counter, command.trim()));
}

fn update_job_counter(shell: &mut Shell) {
    shell.next_job_id = shell.jobs.last().map(|j| j.id + 1).unwrap_or(1);
}

fn main() {
    let mut shell = Shell {
        jobs: Vec::new(),
        next_job_id: 1,
        history: Vec::new(),
        vars: HashMap::new(),
    };

    let mut history_counter = 0;

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let line = match rl.readline("$ ") {
            Ok(line) => line,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        update_history(&mut shell, &mut rl, &line, &mut history_counter);

        let mut commands = parse_commands(&line);

        for cmd in &mut commands {
            expand_command(cmd, &shell);
        }

        update_job_counter(&mut shell);

        if commands.len() == 1 {
            match run(&commands[0], &mut shell) {
                ShellAction::Exit => break,
                ShellAction::Continue => {}
            }
        } else {
            run_pipeline(&commands, &mut shell);
        }

        reap(&mut shell);
    }
}
