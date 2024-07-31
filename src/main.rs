use std::env;
use std::fs;
use std::io::{self, Write};

use scanner::Scanner;

mod scanner;
mod token;
mod token_type;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                return String::new();
            });

            let mut scanner = Scanner::new(file_contents);
            let tokens = scanner.scan_tokens();
            for token in tokens {
                println!("{}", token.to_string());
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}

pub fn error(line: u64, message: String) {
    report(line, "".to_string(), message);
}

fn report(line: u64, r#where: String, message: String) {
    println!("[line {}] Error{}: {}", line, r#where, message);
}
