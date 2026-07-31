use std::{
    io::{self, Write},
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};

use crate::{runner::find_exec, tokenizer::ParsedCmd};

fn build_command(parsed: &ParsedCmd) -> Option<Command> {
    let path = find_exec(&parsed.cmd)?;

    let mut command = Command::new(path);
    command.arg0(&parsed.cmd);
    command.args(&parsed.args);

    Some(command)
}

pub fn run_pipeline(commands: &[ParsedCmd]) {
    let mut err = io::stderr();

    if commands.len() != 2 {
        writeln!(err, "Only two-command pipelines are supported").unwrap();
        return;
    }

    let left = &commands[0];
    let right = &commands[1];

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

