#[allow(unused_imports)]
use std::{
    env,
    fs::{self, Metadata},
    io::{self, Write},
    mem,
    os::unix::fs::PermissionsExt,
    os::unix::process::CommandExt,
    path::{self, PathBuf},
    process::Command,
};

fn is_exec(meta: &Metadata) -> bool {
    meta.permissions().mode() & 0o111 != 0
}

pub fn find_exec(cmd: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH").unwrap();
    for dir in env::split_paths(&path) {
        let candidate = dir.join(cmd);
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

fn parse_command(line: &str) -> (String, Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_db_quotes = false;
    let mut back = false;

    for c in line.chars() {
        if back {
            current.push(c);
            back = false;
            continue;
        }
        match c {
            '\\' => {
                if in_quotes {
                    current.push(c);
                } else {
                    back = true;
                }
            }
            '\'' => {
                if !in_db_quotes {
                    in_quotes = !in_quotes;
                } else {
                    current.push(c);
                }
            }
            '\"' => {
                if !in_quotes {
                    in_db_quotes = !in_db_quotes;
                } else {
                    current.push(c);
                }
            }
            ' ' => {
                if in_quotes || in_db_quotes {
                    current.push(c);
                } else if !current.is_empty() {
                    args.push(mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if back {
        current.push('\\');
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.trim().is_empty() {
            continue;
        }
        let (cmd, args) = parse_command(command.trim());



        match cmd {
            "exit" => break,
            "echo" => println!("{}", args.join(" ")),
            "pwd" => {
                let cwd = env::current_dir().unwrap();
                println!("{}", cwd.display());
            }
            "cd" => {
                if args.is_empty() {
                    println!("cd: No directory specified");
                    continue;
                }
                if args[0] == "~" {
                    env::set_current_dir(env::home_dir().unwrap()).unwrap();
                } else {
                    match env::set_current_dir(&args[0]) {
                        Ok(_) => {}
                        Err(_) => println!("cd: {}: No such file or directory", args[0]),
                    }
                }
            }
            "type" => {
                if args.is_empty() {
                    println!("type: missing argument");
                    continue;
                }
                let arg = &args[0];
                match arg.as_str() {
                    "exit" | "echo" | "type" | "pwd" | "cd" => {
                        println!("{} is a shell builtin", arg)
                    }
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
