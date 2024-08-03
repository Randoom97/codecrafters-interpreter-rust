use std::env;
use std::fs;
use std::io::{self, Write};

use ast_printer::AstPrinter;
use parser::Parser;
use scanner::Scanner;
use token::Token;
use token_type::TokenType;

mod ast;
mod ast_printer;
mod parser;
mod scanner;
mod token;
mod token_type;

static mut HAD_ERROR: bool = false;

pub fn error(line: u64, message: String) {
    report(line, "".to_string(), message);
}

fn report(line: u64, r#where: String, message: String) {
    unsafe { HAD_ERROR = true };
    eprintln!("[line {}] Error{}: {}", line, r#where, message);
}

pub fn error_token(token: &Token, message: String) {
    if token.r#type == TokenType::EOF {
        report(token.line, " at end".to_string(), message);
    } else {
        report(token.line, format!(" at '{}'", token.lexeme), message);
    }
}

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
            let file_contents = read_file(filename);

            let mut scanner = Scanner::new(file_contents);
            let tokens = scanner.scan_tokens();
            for token in tokens {
                println!("{}", token.to_string());
            }

            if unsafe { HAD_ERROR } {
                std::process::exit(65);
            }
        }
        "parse" => {
            let file_contents = read_file(filename);

            let mut scanner = Scanner::new(file_contents);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens.clone());
            let expr = parser.parse();

            if unsafe { HAD_ERROR } {
                std::process::exit(65);
            }

            println!("{}", AstPrinter::new().print(&expr.unwrap()));
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}

fn read_file(filename: &String) -> String {
    return fs::read_to_string(filename).unwrap_or_else(|_| {
        writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
        return String::new();
    });
}
