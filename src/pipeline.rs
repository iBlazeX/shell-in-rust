use std::{
    fs,
    io::{self, Write},
    process::{Child, ChildStdout, Stdio},
};

use crate::{
    builtin::{BuiltinResult, run_builtin},
    execute::{build_command, spawn_external},
    shell::Shell,
    tokenizer::ParsedCmd,
};

fn run_builtin_stage(parsed: &ParsedCmd, shell: &mut Shell) -> io::Result<Vec<u8>> {
    let mut stdout = Vec::new();
    let mut stderr_file;
    let err: &mut dyn Write = if let Some(path) = &parsed.sterr {
        stderr_file = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)?;
        &mut stderr_file
    } else {
        &mut io::stderr()
    };

    match run_builtin(parsed, shell, &mut stdout, err) {
        BuiltinResult::NotBuiltin => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "command is not a builtin",
        )),
        BuiltinResult::Continue | BuiltinResult::Exit => Ok(stdout),
    }
}

fn run_external_stage(parsed: &ParsedCmd, input: Option<Vec<u8>>) -> io::Result<Vec<u8>> {
    let mut command = build_command(parsed)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "command not found"))?;

    if input.is_some() {
        command.stdin(Stdio::piped());
    }

    command.stdout(Stdio::piped());

    if let Some(path) = &parsed.sterr {
        let file = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)?;
        command.stderr(Stdio::from(file));
    }

    let mut child = command.spawn()?;

    if let Some(input) = input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&input)?;
        }
    }

    let output = child.wait_with_output()?;
    Ok(output.stdout)
}

fn run_buffered_pipeline(commands: &[ParsedCmd], shell: &mut Shell) {
    let mut next_input: Option<Vec<u8>> = None;

    for parsed in commands {
        let stage_result = if crate::builtin::is_builtin(&parsed.cmd) {
            run_builtin_stage(parsed, shell)
        } else {
            run_external_stage(parsed, next_input.take())
        };

        match stage_result {
            Ok(stdout) => next_input = Some(stdout),
            Err(_) => {
                writeln!(io::stderr(), "{}: not found", parsed.cmd).unwrap();
                return;
            }
        }
    }

    if let Some(output) = next_input {
        if let Some(last) = commands.last() {
            if let Some(path) = &last.stout {
                let mut file = fs::File::options()
                    .create(true)
                    .write(true)
                    .append(last.append)
                    .truncate(!last.append)
                    .open(path)
                    .unwrap();
                file.write_all(&output).unwrap();
            } else {
                io::stdout().write_all(&output).unwrap();
                io::stdout().flush().unwrap();
            }
        }
    }
}

fn run_external_pipeline(commands: &[ParsedCmd]) {
    let mut children: Vec<Child> = Vec::new();
    let mut previous_stdout: Option<ChildStdout> = None;

    for (i, parsed) in commands.iter().enumerate() {
        let stdin = previous_stdout.take().map(Stdio::from);
        let is_last = i == commands.len() - 1;
        let stdout = if is_last { None } else { Some(Stdio::piped()) };

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

        if !is_last {
            previous_stdout = child.stdout.take();
        }

        children.push(child);
    }

    for mut child in children {
        let _ = child.wait();
    }
}

pub fn run_pipeline(commands: &[ParsedCmd], shell: &mut Shell) {
    if commands.iter().any(|parsed| crate::builtin::is_builtin(&parsed.cmd)) {
        run_buffered_pipeline(commands, shell);
    } else {
        run_external_pipeline(commands);
    }
}
