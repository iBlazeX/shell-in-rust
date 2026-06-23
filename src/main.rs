#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command.trim().to_string();
        match command.as_str() {
            "exit" => break,
            "echo" => println!(command),
            _ => println!("{}: command not found", command.trim()),
        }
    }
}
