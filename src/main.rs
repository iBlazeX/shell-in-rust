use std::fs::Metadata;
#[allow(unused_imports)]
use std::{
    env, fs,
    io::{self, Write},
    mem,
    os::unix::fs::PermissionsExt,
    os::unix::process::CommandExt,
    path::{self, PathBuf},
    process::Command,
};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.trim().is_empty() {
            continue;
        }
        let (cmd, args, stout) = tokenize(command.trim());
        let mut file;
        let out: &mut dyn Write = if let Some(path) = stout {
            file = fs::File::create(path).unwrap();
            &mut file
        } else {
            &mut io::stdout()
        };
        match cmd.as_str() {
            "exit" => break,
            "echo" => writeln!(out, "{}", args.join(" ")).unwrap(),
            "pwd" => {
                let cwd = env::current_dir().unwrap();
                writeln!(out, "{}", cwd.display()).unwrap();
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
                        writeln!(out, "{} is a shell builtin", arg).unwrap();
                    }
                    _ => match find_exec(arg) {
                        Some(path) => writeln!(out, "{} is {}", arg, path.display()).unwrap(),
                        None => println!("{}: not found", arg),
                    },
                }
            }
            "cat" => match fs::read_to_string(&args[0]) {
                Ok(content) => writeln!(out, "{}", content).unwrap(),
                Err(_) => println!("cat: {}: No such file or directory", args[0]),
            },
            _ => match find_exec(cmd.as_str()) {
                Some(path) => {
                    Command::new(path).arg0(cmd).args(args).status().unwrap();
                }
                None => println!("{}: not found", cmd),
            },
        }
    }
}

fn is_exec(meta: &Metadata) -> bool {
    meta.permissions().mode() & 0o111 != 0
}

fn find_exec(cmd: &str) -> Option<PathBuf> {
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

fn tokenize(line: &str) -> (String, Vec<String>, Option<String>) {
    let mut token = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_db_quotes = false;
    let mut backslash = false;
    let mut expect_stdout = false;
    let mut stout = None;

    for c in line.chars() {
        if backslash {
            current.push(c);
            backslash = false;
            continue;
        }
        match c {
            '\\' => {
                if in_quotes {
                    current.push(c);
                } else {
                    backslash = true;
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
                    if expect_stdout {
                        stout = Some(mem::take(&mut current));
                        expect_stdout = false;
                    } else {
                        token.push(mem::take(&mut current));
                    }
                }
            }
            '>' => {
                if !in_quotes && !in_db_quotes && !backslash {
                    if !current.is_empty() {
                        token.push(mem::take(&mut current));
                    }
                    expect_stdout = true;
                } else {
                    current.push(c);
                }
            }
            _ => current.push(c),
        }
    }
    if backslash {
        current.push('\\');
    }
    if expect_stdout {
        stout = Some(current);
    } else if !current.is_empty() {
        token.push(current);
    }
    let cmd = token.remove(0);
    (cmd, token, stout)
}
