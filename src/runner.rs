use crate::tokenizer::ParsedCmd;
use crate::{Shell, jobs::Job};
use std::{
    env,
    fs::{self, Metadata},
    io::Write,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::PathBuf,
    process::Command,
};

pub enum ShellAction {
    Continue,
    Exit,
}

pub fn run(
    parsed: &ParsedCmd,
    shell: &mut Shell,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> ShellAction {
    let ParsedCmd {
        cmd,
        args,
        stout,
        sterr,
        append,
        bg,
    } = parsed;
    if parsed.bg {
        run_external(cmd, args, sterr, stout, err, append, bg, shell);
    } else {
        match cmd.as_str() {
            "exit" => return ShellAction::Exit,
            "echo" => echo(args, out),
            "pwd" => pwd(out),
            "cd" => cd(args, err),
            "type" => type_cmd(args, out, err),
            "cat" => cat(args, out, err),
            "jobs" => {
                for job in &shell.jobs {
                    println!(
                        "[{}]+  {:?}                 {} &",
                        job.id, job.status, job.token,
                    )
                }
            }
            _ => run_external(cmd, args, sterr, stout, err, append, bg, shell),
        }
    }
    ShellAction::Continue
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
fn echo(args: &Vec<String>, out: &mut dyn Write) {
    writeln!(out, "{}", args.join(" ")).unwrap();
}
fn pwd(out: &mut dyn Write) {
    let cwd = env::current_dir().unwrap();
    writeln!(out, "{}", cwd.display()).unwrap();
}
fn cd(args: &Vec<String>, err: &mut dyn Write) {
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
fn type_cmd(args: &Vec<String>, out: &mut dyn Write, err: &mut dyn Write) {
    if args.is_empty() {
        writeln!(out, "type: missing argument").unwrap();
        return;
    }
    let arg = &args[0];
    match arg.as_str() {
        "exit" | "echo" | "type" | "pwd" | "cd" | "jobs" => {
            writeln!(out, "{} is a shell builtin", arg).unwrap();
        }
        _ => match find_exec(arg) {
            Some(path) => writeln!(out, "{} is {}", arg, path.display()).unwrap(),
            None => writeln!(err, "{}: not found", arg).unwrap(),
        },
    }
}
fn cat(args: &Vec<String>, out: &mut dyn Write, err: &mut dyn Write) {
    for file in args {
        match fs::read_to_string(file) {
            Ok(content) => write!(out, "{}", content).unwrap(),
            Err(_) => writeln!(err, "cat: {}: No such file or directory", file).unwrap(),
        }
    }
}
fn run_external(
    cmd: &str,
    args: &Vec<String>,
    sterr: &Option<String>,
    stout: &Option<String>,
    err: &mut dyn Write,
    append: &bool,
    bg: &bool,
    shell: &mut Shell,
) {
    match find_exec(cmd) {
        Some(path) => {
            let mut command = Command::new(path);
            command.arg0(cmd).args(args);

            if let Some(path) = stout {
                let file = fs::File::options()
                    .create(true)
                    .write(true)
                    .append(*append)
                    .truncate(!*append)
                    .open(path)
                    .unwrap();

                command.stdout(file);
            }
            if let Some(path) = &sterr {
                let file = fs::File::options()
                    .create(true)
                    .write(true)
                    .append(*append)
                    .truncate(!*append)
                    .open(path)
                    .unwrap();
                command.stderr(file);
            }
            if *bg {
                let child = command.spawn().unwrap();
                println!("[{}] {}", shell.next_job_id, child.id());
                shell.jobs.push(Job {
                    id: shell.next_job_id,
                    child,
                    token: String::from(cmd) + " " + &args.join(" "),
                    status: crate::jobs::JobStatus::Running,
                });
                shell.next_job_id += 1;
            } else {
                command.status().unwrap();
            }
        }
        None => writeln!(err, "{}: not found", cmd).unwrap(),
    }
}
