#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        println!(_ => println!("{}", command.trim());
        match command.trim() {
            "exit" => break,
            "echo" => println!("{}", &command[5..]),
            _ => println!("{}: command not found", command.trim()),
        }
    }
}
