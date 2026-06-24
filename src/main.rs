use ::std::env;
use std::fs::Metadata;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path;
use std::{env::split_paths, fs::metadata, path::PathBuf};

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
        let meta = metadata(&candidate).unwrap();
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
        let (cmd, args) = command
            .trim()
            .split_once(' ')
            .unwrap_or((command.trim(), ""));
        match cmd {
            "exit" => break,
            "echo" => println!("{}", args),
            "type" => match args {
                "exit" | "echo" | "type" => println!("{} is a shell builtin", args),
                _ => match find_exec(args) {
                    Some(path) => println!("{} is {}", args, path.display()),
                    None => println!("{}: not found", args),
                },
            },
            _ => println!("{}: command not found", cmd),
        }
    }
}
