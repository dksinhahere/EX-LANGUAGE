use crate::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Built-in commands
    Chd,
    Cud,
    Clean,
    Exit,
    AndAnd,
    ExecMarker, // >>
    
    // Identifiers and literals
    Word(String),
    LocalPath(String), // ./something
    
    // End of input
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
}

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }

            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
        });

        tokens
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() 
            && self.current_char().is_whitespace() {
            self.position += 1;
        }
    }

    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn peek(&self, offset: usize) -> char {
        self.input.chars().nth(self.position + offset).unwrap_or('\0')
    }

    fn next_token(&mut self) -> Option<Token> {
        // Check for >> (exec marker)
        if self.current_char() == '>' && self.peek(1) == '>' {
            self.position += 2;
            return Some(Token {
                token_type: TokenType::ExecMarker,
                lexeme: ">>".to_string(),
            });
        }

        // Check for local path (./)
        if self.current_char() == '.' && self.peek(1) == '/' {
            return Some(self.read_local_path());
        }

        if self.current_char() == '&' && self.peek(1) == '&' {
            self.position += 2;
            return Some(Token {
                token_type: TokenType::AndAnd,
                lexeme: "&&".to_string()
            });
        }

        // Read word
        let word = self.read_word();
        if word.is_empty() {
            return None;
        }

        // Match keywords
        let token_type = match word.as_str() {
            "chd" => TokenType::Chd,
            "cud" => TokenType::Cud,
            "clean" => TokenType::Clean,
            "exit" => TokenType::Exit,
            _ => TokenType::Word(word.clone()),
        };

        Some(Token {
            token_type,
            lexeme: word,
        })
    }

    fn read_word(&mut self) -> String {
        let start = self.position;
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() {
                break;
            }
            self.position += 1;
        }

        self.input[start..self.position].to_string()
    }

    fn read_local_path(&mut self) -> Token {
        let start = self.position;
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() {
                break;
            }
            self.position += 1;
        }

        let path = self.input[start..self.position].to_string();
        Token {
            token_type: TokenType::LocalPath(path.clone()),
            lexeme: path,
        }
    }
}