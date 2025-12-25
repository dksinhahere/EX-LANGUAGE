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

fn run_source(source: &str) {
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
    let program = parser.parse();

    // 3) Interpret (interpreter updates environment internally)
    let mut interp = Interpreter::new();
    if let Err(e) = interp.accept(&program) {
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

    // lex + parse file content
    run_source(&source);
}

fn main() {
    loop {
        match env::current_dir() {
            Ok(path) => print!("{}> ", path.display()),
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
                            run_source(&command);
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

                            run_source(&source);
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
