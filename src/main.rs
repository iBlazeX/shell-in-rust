use std::process::Command;
#[allow(unused_imports)]
use std::{
    env,
    fs::{self, Metadata},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{self, PathBuf},
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
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        let (cmd, args) = (parts[0], &parts[1..]);
        match cmd {
            "exit" => break,
            "echo" => println!("{}", args.join(" ")),
            "type" => {
                let arg = args[0];
                match arg {
                    "exit" | "echo" | "type" => println!("{} is a shell builtin", arg),
                    _ => match find_exec(arg) {
                        Some(path) => println!("{} is {}", arg, path.display()),
                        None => println!("{}: not found", arg),
                    },
                }
            }
            _ => match find_exec(cmd) {
                Some(path) => {
                    Command::new(path).status().unwrap();
                }
                None => println!("{}: not found", cmd),
            },
        }
    }
}
