use super::lexer::{Token, TokenType};

#[derive(Debug)]
pub enum Command {
    Chd { directory: String },
    Cud,
    Clean,
    Exit,
    ExecCode { code: String },
    LocalExec { path: String, args: Vec<String> },
    SystemCommand { command: String, args: Vec<String> },
    Chain { commands: Vec<Command> }, // New variant for chained commands
    Empty,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Command, String> {
        if self.is_at_end() {
            return Ok(Command::Empty);
        }

        // Parse first command
        let first_cmd = self.parse_single_command()?;
        
        // Check if there are chained commands
        let mut commands = vec![first_cmd];
        
        while self.match_token(TokenType::AndAnd) {
            let next_cmd = self.parse_single_command()?;
            commands.push(next_cmd);
        }
        
        // If only one command, return it directly
        if commands.len() == 1 {
            Ok(commands.into_iter().next().unwrap())
        } else {
            Ok(Command::Chain { commands })
        }
    }

    fn parse_single_command(&mut self) -> Result<Command, String> {
        if self.is_at_end() {
            return Ok(Command::Empty);
        }

        let token = self.advance();

        match &token.token_type {
            TokenType::Exit => Ok(Command::Exit),
            
            TokenType::Chd => {
                if let Some(dir_token) = self.peek_token() {
                    if let TokenType::Word(dir) = &dir_token.token_type {
                        let directory = dir.clone();
                        self.advance();
                        Ok(Command::Chd { directory })
                    } else {
                        Err("Expected directory path after 'chd'".to_string())
                    }
                } else {
                    Err("No directory specified for 'chd'".to_string())
                }
            }
            
            TokenType::Cud => Ok(Command::Cud),
            
            TokenType::Clean => Ok(Command::Clean),
            
            TokenType::ExecMarker => {
                let mut code_parts = Vec::new();
                
                while !self.is_at_end() {
                    if let Some(tok) = self.peek_token() {
                        if tok.token_type == TokenType::Eof 
                            || tok.token_type == TokenType::AndAnd {
                            break;
                        }
                        code_parts.push(tok.lexeme.clone());
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                let code = code_parts.join(" ");
                Ok(Command::ExecCode { code })
            }
            
            TokenType::LocalPath(path) => {
                let mut args = Vec::new();
                
                while !self.is_at_end() {
                    if let Some(tok) = self.peek_token() {
                        if tok.token_type == TokenType::Eof 
                            || tok.token_type == TokenType::AndAnd {
                            break;
                        }
                        if let TokenType::Word(word) = &tok.token_type {
                            args.push(word.clone());
                        } else {
                            args.push(tok.lexeme.clone());
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                Ok(Command::LocalExec {
                    path: path.clone(),
                    args,
                })
            }
            
            TokenType::Word(cmd) => {
                let command = cmd.clone();
                let mut args = Vec::new();
                
                while !self.is_at_end() {
                    if let Some(tok) = self.peek_token() {
                        if tok.token_type == TokenType::Eof 
                            || tok.token_type == TokenType::AndAnd {
                            break;
                        }
                        if let TokenType::Word(word) = &tok.token_type {
                            args.push(word.clone());
                        } else {
                            args.push(tok.lexeme.clone());
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                Ok(Command::SystemCommand { command, args })
            }
            
            TokenType::Eof => Ok(Command::Empty),
            TokenType::AndAnd => Err("Unexpected '&&' operator".to_string()),
        }
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if let Some(tok) = self.peek_token() {
            if tok.token_type == token_type {
                self.advance();
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens[self.current - 1].clone()
    }

    fn peek_token(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            Some(&self.tokens[self.current])
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() 
            || self.tokens[self.current].token_type == TokenType::Eof
    }
}