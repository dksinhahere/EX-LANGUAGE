use libc::geteuid;
use std::env;
use std::io::{self, Write};

mod interpreter;
mod lexer;
mod local;
mod parser;
mod values;

use crate::local::{CommandInterpreter, Lexer as LocalLexer, Parser as LocalParser};

fn is_root() -> bool {
    unsafe { geteuid() == 0 }
}

fn main() {
    let mut cmd_interp = CommandInterpreter::new();

    loop {
        let which = if is_root() { "#" } else { "%" };

        match env::current_dir() {
            Ok(path) => print!(
                "PATH=[{}] USER=[{}]{} ",
                path.display(),
                whoami::username(),
                which
            ),
            Err(_) => print!("?> "),
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                // Tokenize input
                let mut lexer = LocalLexer::new(input.to_string());
                let tokens = lexer.tokenize();

                // Parse tokens into command
                let mut parser = LocalParser::new(tokens);
                let command = match parser.parse() {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                        continue;
                    }
                };

                // Execute command
                match cmd_interp.execute(command) {
                    local::interpreter::ExecutionResult::Exit => break,
                    local::interpreter::ExecutionResult::Continue => {}
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
            }
        }
    }
}
