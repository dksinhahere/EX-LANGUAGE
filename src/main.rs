use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

mod interpreter;
mod lexer;
mod parser;
mod values;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

fn run_source(source: &str, interp: &mut Interpreter) {
    // Pass interpreter as parameter
    // 1) Lex
    let tokens = match Lexer::new(source.to_string()).scan_tokens() {
        Ok(t) => t,
        Err(e) => {
            e.display(source);
            return;
        }
    };

    // 2) Parse
    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(stmts) => stmts,
        Err(errors) => {
            for err in errors {
                eprintln!(
                    "[line {}] Error at '{}': {}",
                    err.token.line, err.token.lexeme, err.message
                );
            }
            return;
        }
    };

    // 3) Interpret
    if let Err(e) = interp.interpret(&statements) {
        eprintln!("Runtime error: {e}");
    }
}

fn run_ex_file(path_str: &str) {
    let path = Path::new(path_str);

    // check exists
    if !path.exists() {
        eprintln!("File not found: {}", path_str);
        return;
    }

    // check extension .ex
    match path.extension().and_then(|e| e.to_str()) {
        Some("ex") => {}
        _ => {
            eprintln!("exsh only supports .ex files. Got: {}", path_str);
            return;
        }
    }

    // read file
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {}: {}", path_str, e);
            return;
        }
    };

    // Create new interpreter for file execution
    let mut interp = Interpreter::new();
    run_source(&source, &mut interp);
}

fn main() {
    let mut interp = Interpreter::new(); // Create interpreter once at start

    loop {
        match env::current_dir() {
            Ok(path) => print!("PATH=[{}] USER=[{}]% ", path.display(), whoami::username()),
            Err(_) => print!("?> "),
        }
        io::stdout().flush().unwrap();

        let mut input: String = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input == "exit" {
                    break;
                }

                let mut parts = input.split_whitespace();
                match parts.next() {
                    Some("chd") => {
                        if let Some(dir) = parts.next() {
                            if let Err(e) = env::set_current_dir(dir) {
                                eprintln!("Error changing directory: {}", e);
                            }
                        } else {
                            eprintln!("No directory specified");
                        }
                    }
                    Some("cud") => match env::current_dir() {
                        Ok(path) => println!("{}", path.display()),
                        Err(e) => eprintln!("Error getting current directory: {}", e),
                    },
                    Some("clean") => {
                        print!("\x1B[2J\x1B[1;1H");
                    }
                    Some(">>") => {
                        let command: String = parts.collect::<Vec<&str>>().join(" ");
                        if !command.is_empty() {
                            run_source(&command, &mut interp); // Pass the persistent interpreter
                        }
                    }
                    Some("exsh") => {
                        if let Some(file) = parts.next() {
                            let path = Path::new(file);

                            if path.extension().and_then(|s| s.to_str()) != Some("ex") {
                                eprintln!("Usage: exsh <file.ex>");
                                continue;
                            }

                            let source = match fs::read_to_string(path) {
                                Ok(s) => s,
                                Err(e) => {
                                    eprintln!("Error reading file: {e}");
                                    continue;
                                }
                            };

                            interp = Interpreter::new();
                            run_source(&source, &mut interp); // Use the same interpreter
                        } else {
                            eprintln!("Usage: exsh <file.ex>");
                        }
                    }
                    Some(cmd) => {
                        eprintln!("Unknown command: {}", cmd);
                    }
                    None => {}
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
            }
        }
    }
}
