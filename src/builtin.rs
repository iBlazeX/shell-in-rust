use crate::{jobs::JobStatus, shell::Shell, tokenizer::ParsedCmd};
use std::{
    env,
    fs::{self},
    io::Write,
};

pub enum BuiltinResult {
    NotBuiltin,
    Continue,
    Exit,
}

pub fn run_builtin(
    parsed: &ParsedCmd,
    shell: &mut Shell,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> BuiltinResult {
    match parsed.cmd.as_str() {
        "exit" => BuiltinResult::Exit,

        "echo" => {
            echo(&parsed.args, out);
            BuiltinResult::Continue
        }

        "pwd" => {
            pwd(out);
            BuiltinResult::Continue
        }

        "cd" => {
            cd(&parsed.args, err);
            BuiltinResult::Continue
        }

        "type" => {
            type_cmd(&parsed.args, out, err);
            BuiltinResult::Continue
        }

        "jobs" => {
            jobs(shell);
            BuiltinResult::Continue
        }

        "history" => {
            history(out, shell, &parsed.args);
            BuiltinResult::Continue
        }

        "declare" => {
            declare(&parsed.args, out, err, shell);
            BuiltinResult::Continue
        }

        _ => BuiltinResult::NotBuiltin,
    }
}

fn echo(args: &[String], out: &mut dyn Write) {
    writeln!(out, "{}", args.join(" ")).unwrap();
}

fn pwd(out: &mut dyn Write) {
    let cwd = env::current_dir().unwrap();
    writeln!(out, "{}", cwd.display()).unwrap();
}

fn cd(args: &[String], err: &mut dyn Write) {
    if args.is_empty() {
        writeln!(err, "cd: No directory specified").unwrap();
        return;
    }

    if args[0] == "~" {
        env::set_current_dir(env::home_dir().unwrap()).unwrap();
    } else {
        match env::set_current_dir(&args[0]) {
            Ok(_) => {}
            Err(_) => writeln!(err, "cd: {}: No such file or directory", args[0]).unwrap(),
        }
    }
}

fn type_cmd(args: &[String], out: &mut dyn Write, err: &mut dyn Write) {
    if args.is_empty() {
        writeln!(out, "type: missing argument").unwrap();
        return;
    }

    let arg = &args[0];

    match arg.as_str() {
        "exit" | "echo" | "pwd" | "cd" | "type" | "jobs" | "history" | "declare" => {
            writeln!(out, "{} is a shell builtin", arg).unwrap();
        }
        _ => match crate::execute::find_exec(arg) {
            Some(path) => {
                writeln!(out, "{} is {}", arg, path.display()).unwrap();
            }
            None => {
                writeln!(err, "{}: not found", arg).unwrap();
            }
        },
    }
}

fn cat(args: &[String], out: &mut dyn Write, err: &mut dyn Write) {
    for file in args {
        match fs::read_to_string(file) {
            Ok(content) => write!(out, "{}", content).unwrap(),
            Err(_) => writeln!(err, "cat: {}: No such file or directory", file).unwrap(),
        }
    }
}

fn jobs(shell: &mut Shell) {
    let len = shell.jobs.len();

    for (i, job) in shell.jobs.iter_mut().enumerate() {
        if job.child.try_wait().unwrap().is_some() {
            job.status = JobStatus::Done;
        }

        let marker = match i {
            x if x + 1 == len => "+",
            x if x + 2 == len => "-",
            _ => " ",
        };

        println!(
            "[{}]{}  {:?}                 {}{}",
            job.id,
            marker,
            job.status,
            job.token,
            if job.status == JobStatus::Running {
                " &"
            } else {
                ""
            }
        );
    }

    shell.jobs.retain(|job| job.status != JobStatus::Done);
}

fn history(out: &mut dyn Write, shell: &mut Shell, args: &[String]) {
    let n = if let Some(arg) = args.first() {
        arg.parse::<usize>().unwrap()
    } else {
        shell.history.len()
    };

    let start = shell.history.len().saturating_sub(n);

    for cmd in &shell.history[start..] {
        writeln!(out, "{cmd}").unwrap();
    }
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();

    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }

    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn declare(args: &[String], out: &mut dyn Write, err: &mut dyn Write, shell: &mut Shell) {
    if args.first().map(String::as_str) == Some("-p") {
        if let Some(name) = args.get(1) {
            if let Some(value) = shell.vars.get(name) {
                writeln!(out, "declare -- {}=\"{}\"", name, value).unwrap();
            } else {
                writeln!(err, "declare: {}: not found", name).unwrap();
            }
        }
        return;
    }

    if let Some(arg) = args.first() {
        if let Some((name, value)) = arg.split_once('=') {
            if valid_identifier(name) {
                shell.vars.insert(name.to_string(), value.to_string());
            } else {
                writeln!(err, "declare: `{}': not a valid identifier", arg).unwrap();
            }
        }
    }
}

pub fn is_builtin(cmd: &str) -> bool {
    matches!(
        cmd,
        "exit" | "echo" | "pwd" | "cd" | "type" | "jobs" | "history" | "declare"
    )
}
