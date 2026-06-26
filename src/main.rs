mod runner;
mod tokenizer;
#[allow(unused_imports)]
use runner::{ShellAction, run};
use std::fs::Metadata;
use std::{os::unix::fs::PermissionsExt, os::unix::process::CommandExt};
use tokenizer::tokenize;

fn main() {
    loop {
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
            file = fs::File::create(path).unwrap();
            &mut file
        } else {
            &mut io::stdout()
        };
        let err: &mut dyn Write = if let Some(path) = &parsed.sterr {
            errfile = fs::File::create(path).unwrap();
            &mut errfile
        } else {
            &mut io::stderr()
        };
        match run(&parsed, out, err) {
            ShellAction::Exit => break,
            ShellAction::Continue => {}
        }
    }
}
