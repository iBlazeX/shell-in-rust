#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let (cmd, args) = command
            .split_once([' ', '\n'])
            .unwrap_or((command.as_str(), ""));
        match cmd {
            "exit" => break,
            "echo" => print!("{}", args),
            "type" => match args {
                "exit" | "echo" | "type" => print!("{} is a shell extension", args),
                _ => println!("{}: not found", args),
            },
            _ => println!("{}: command not found", cmd),
        }
    }
}
