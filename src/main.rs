use std::env;
use std::fs;
use std::io::{self, Write};

use ast_printer::AstPrinter;
use expr::Expr;
use interpreter::Interpreter;
use interpreter::RuntimeError;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;
use stmt::Stmt;
use token::Token;
use token_type::TokenType;

mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod lox_callables;
mod lox_instance;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_type;

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

pub fn error(line: u64, message: &str) {
    report(line, "".to_string(), message);
}

fn report(line: u64, r#where: String, message: &str) {
    unsafe { HAD_ERROR = true };
    eprintln!("[line {}] Error{}: {}", line, r#where, message);
}

pub fn error_token(token: &Token, message: &str) {
    if token.r#type == TokenType::EOF {
        report(token.line, " at end".to_string(), message);
    } else {
        report(token.line, format!(" at '{}'", token.lexeme), message);
    }
}

pub fn runtime_error(error: RuntimeError) {
    eprintln!("{}\n[line {}]", error.message, error.token.line);
    unsafe { HAD_RUNTIME_ERROR = true };
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
            let tokens = tokenize(filename);
            for token in tokens {
                println!("{}", token.to_string());
            }

            if unsafe { HAD_ERROR } {
                std::process::exit(65);
            }
        }
        "parse" => {
            let expr = parse_expr(filename);

            if unsafe { HAD_ERROR } {
                std::process::exit(65);
            }

            println!("{}", AstPrinter::new().print(&expr.unwrap()));
        }
        "evaluate" => {
            let expr = parse_expr(filename);

            if unsafe { HAD_ERROR } {
                std::process::exit(65);
            }

            Interpreter::new().interpret_expr(expr.unwrap());

            if unsafe { HAD_RUNTIME_ERROR } {
                std::process::exit(70);
            }
        }
        "run" => {
            let statement_options = parse(filename);

            if unsafe { HAD_ERROR } {
                std::process::exit(65);
            }

            // would have had errors, and exited, if any of the options were None
            let statements: Vec<Stmt> = statement_options.into_iter().flatten().collect();

            let interpreter = Interpreter::new();
            let mut resolver = Resolver::new(interpreter);
            resolver.resolve_stmts(&statements);

            if unsafe { HAD_ERROR } {
                std::process::exit(65);
            }

            resolver.interpreter.interpret(statements);

            if unsafe { HAD_RUNTIME_ERROR } {
                std::process::exit(70);
            }
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

fn tokenize(filename: &String) -> Vec<Token> {
    let file_contents = read_file(filename);

    let mut scanner = Scanner::new(file_contents);
    return scanner.scan_tokens().clone();
}

fn parse_expr(filename: &String) -> Option<Expr> {
    let tokens = tokenize(filename);
    return Parser::new(tokens.clone()).parse_expr();
}

fn parse(filename: &String) -> Vec<Option<Stmt>> {
    let tokens = tokenize(filename);
    return Parser::new(tokens.clone()).parse();
}
