#[allow(unused_imports)]
use std::io::{self, Write};

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
                _ => println!("{}: not found", args),
            },
            _ => println!("{}: command not found", cmd),
        }
    }
}
