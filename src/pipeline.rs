use std::{
    io::{self, Write},
    process::{Child, ChildStdout, Stdio},
};

use crate::{execute::spawn_external, shell::Shell, tokenizer::ParsedCmd};

pub fn run_pipeline(commands: &[ParsedCmd], _shell: &mut Shell) {
    let mut children: Vec<Child> = Vec::new();
    let mut previous_stdout: Option<ChildStdout> = None;

    for (i, parsed) in commands.iter().enumerate() {
        let stdin = previous_stdout.take().map(Stdio::from);

        let stdout = if i == commands.len() - 1 {
            None
        } else {
            Some(Stdio::piped())
        };

        let mut child = match spawn_external(parsed, stdin, stdout, None) {
            Ok(child) => child,
            Err(_) => {
                writeln!(io::stderr(), "{}: not found", parsed.cmd).unwrap();

                for mut child in children {
                    let _ = child.wait();
                }

                return;
            }
        };

        previous_stdout = child.stdout.take();

        children.push(child);
    }

    for mut child in children {
        child.wait().unwrap();
    }
}
