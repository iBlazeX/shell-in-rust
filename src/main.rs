#[allow(unused_imports)]
use std::{
    env,
    fs::{self, Metadata},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    os::unix::process::CommandExt,
    path::{self, PathBuf},
    process::Command,
};

fn is_exec(meta: &Metadata) -> bool {
    meta.permissions().mode() & 0o111 != 0
}

pub fn find_exec(args: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH").unwrap();
    for dir in env::split_paths(&path) {
        let candidate = dir.join(args);
        if !candidate.is_file() {
            continue;
        }
        let meta = fs::metadata(&candidate).unwrap();
        if is_exec(&meta) {
            return Some(candidate);
        }
    }
    None
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let (cmd, parts) = command
            .trim()
            .split_once(' ')
            .unwrap_or((command.trim(), ""));
        let parts = parts.trim();
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for c in args.chars() {
            match c {
                '\'' => in_quotes = !in_quotes,
                ' ' => {
                    if in_quotes {
                        current.push(c);
                    } else {
                        args.push(current);
                        current.clear();
                    }
                }
                _ => current.push(c),
            }
        }

        match cmd {
            "exit" => break,
            "echo" => println!("{}", args.join(" ")),
            "pwd" => {
                let cwd = env::current_dir().unwrap();
                println!("{}", cwd.display());
            }
            "cd" => {
                if (args[0] == "~") {
                    env::set_current_dir(env::home_dir().unwrap());
                } else {
                    match env::set_current_dir(args[0]) {
                        Ok(_) => {}
                        Err(_) => println!("cd: {}: No such file or directory", args[0]),
                    }
                }
            }
            "type" => {
                let arg = args[0];
                match arg {
                    "exit" | "echo" | "type" | "pwd" => println!("{} is a shell builtin", arg),
                    _ => match find_exec(arg) {
                        Some(path) => println!("{} is {}", arg, path.display()),
                        None => println!("{}: not found", arg),
                    },
                }
            }
            _ => match find_exec(cmd) {
                Some(path) => {
                    Command::new(path).arg0(cmd).args(args).status().unwrap();
                }
                None => println!("{}: not found", cmd),
            },
        }
    }
}
