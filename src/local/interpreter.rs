use super::parser::Command;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

pub enum ExecutionResult {
    Continue,
    Exit,
}

pub struct CommandInterpreter {
    ex_interpreter: Interpreter,
}

impl CommandInterpreter {
    pub fn new() -> Self {
        Self {
            ex_interpreter: Interpreter::new(),
        }
    }

    pub fn execute(&mut self, command: Command) -> ExecutionResult {
        match command {
            Command::Exit => ExecutionResult::Exit,
            
            Command::Chd { directory } => {
                if let Err(e) = env::set_current_dir(&directory) {
                    eprintln!("Error changing directory: {}", e);
                }
                ExecutionResult::Continue
            }
            
            Command::Cud => {
                match env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => eprintln!("Error getting current directory: {}", e),
                }
                ExecutionResult::Continue
            }
            
            Command::Clean => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush().unwrap();
                ExecutionResult::Continue
            }
            
            Command::ExecCode { code } => {
                if !code.is_empty() {
                    self.run_ex_source(&code);
                }
                ExecutionResult::Continue
            }
            
            Command::LocalExec { path, args } => {
                self.execute_local_path(&path, &args);
                ExecutionResult::Continue
            }
            
            Command::SystemCommand { command, args } => {
                self.execute_system_command(&command, &args);
                ExecutionResult::Continue
            }
            
            Command::Empty => ExecutionResult::Continue,
        }
    }

    fn run_ex_source(&mut self, source: &str) {
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
        if let Err(e) = self.ex_interpreter.interpret(&statements) {
            eprintln!("Runtime error: {e}");
        }
    }

    fn execute_local_path(&mut self, path: &str, args: &[String]) {
        let path_obj = Path::new(path);

        // Check if it's a .ex file
        if path_obj.extension().and_then(|s| s.to_str()) == Some("ex") {
            self.run_ex_file(path);
        } else {
            // Try to execute as a local binary/script
            match process::Command::new(path).args(args).status() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("Process exited with status: {}", status);
                    }
                }
                Err(e) => eprintln!("Failed to execute {}: {}", path, e),
            }
        }
    }

    fn run_ex_file(&mut self, path_str: &str) {
        let path = Path::new(path_str);

        // Check exists
        if !path.exists() {
            eprintln!("File not found: {}", path_str);
            return;
        }

        // Check extension .ex
        match path.extension().and_then(|e| e.to_str()) {
            Some("ex") => {}
            _ => {
                eprintln!("exsh only supports .ex files. Got: {}", path_str);
                return;
            }
        }

        // Read file
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading file {}: {}", path_str, e);
                return;
            }
        };

        self.run_ex_source(&source);
    }

    fn execute_system_command(&self, command: &str, args: &[String]) {
        match process::Command::new(command).args(args).status() {
            Ok(status) => {
                if !status.success() {
                    eprintln!("Process exited with status: {}", status);
                }
            }
            Err(_) => {
                eprintln!("Unknown command or failed to execute: {}", command)
            }
        }
    }
}