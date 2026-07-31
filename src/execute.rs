use std::{
    env,
    fs::{self, Metadata},
    io,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::tokenizer::ParsedCmd;

fn is_exec(meta: &Metadata) -> bool {
    meta.permissions().mode() & 0o111 != 0
}

pub fn find_exec(cmd: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for dir in env::split_paths(&path) {
        let candidate = dir.join(cmd);

        if !candidate.is_file() {
            continue;
        }

        let meta = fs::metadata(&candidate).ok()?;

        if is_exec(&meta) {
            return Some(candidate);
        }
    }

    None
}

pub fn build_command(parsed: &ParsedCmd) -> Option<Command> {
    let path = find_exec(&parsed.cmd)?;

    let mut command = Command::new(path);

    command.arg0(&parsed.cmd);
    command.args(&parsed.args);

    Some(command)
}

pub fn spawn_external(
    parsed: &ParsedCmd,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
) -> io::Result<Child> {
    let mut command = build_command(parsed)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "command not found"))?;

    if let Some(stdin) = stdin {
        command.stdin(stdin);
    }

    if let Some(stdout) = stdout {
        command.stdout(stdout);
    } else if let Some(path) = &parsed.stout {
        let file = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)?;

        command.stdout(Stdio::from(file));
    }

    if let Some(stderr) = stderr {
        command.stderr(stderr);
    } else if let Some(path) = &parsed.sterr {
        let file = fs::File::options()
            .create(true)
            .write(true)
            .append(parsed.append)
            .truncate(!parsed.append)
            .open(path)?;

        command.stderr(Stdio::from(file));
    }

    command.spawn()
}
