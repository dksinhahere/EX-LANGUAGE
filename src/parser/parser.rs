use crate::lexer::{Token, TokenKind};
use crate::parser::ast::{Expr, Literal, Stmt};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    // =========================================================
    // Constructor
    // =========================================================

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    // =========================================================
    // Entry point
    // =========================================================

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParseError>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => self.synchronize(),
            }
        }

        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.errors.clone())
        }
    }

    // =========================================================
    // Declarations & Statements
    // =========================================================

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        self.statement()
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().kind {

            TokenKind::VLock => {
                self.advance();
                let identifier: String = self.consume(TokenKind::Identifier, "Expected 'Identifier'").unwrap().lexeme;
                Ok(Stmt::SmartLock {variable: identifier})
            }

            TokenKind::VUnlock => {
                self.advance();
                let identifier: String = self.consume(TokenKind::Identifier, "Expected 'Identifier'").unwrap().lexeme;
                Ok(Stmt::SmartUnlock {variable: identifier})
            }

            TokenKind::VKill => {
                self.advance();
                let identifier: String = self.consume(TokenKind::Identifier, "Expected 'Identifier'").unwrap().lexeme;
                Ok(Stmt::SmartKill {variable: identifier})
            }
            
            TokenKind::VRevive => {
                self.advance();
                let identifier: String = self.consume(TokenKind::Identifier, "Expected 'Identifier'").unwrap().lexeme;
                Ok(Stmt::SmartRevive {variable: identifier})
            }

            TokenKind::VConst => {
                self.advance();
                let identifier: String = self.consume(TokenKind::Identifier, "Expected 'Identifier'").unwrap().lexeme;
                Ok(Stmt::SmartConst {variable: identifier})
            }
            TokenKind::Label => {
                self.advance();
                return self.consume_label();
            }
            TokenKind::If => {
                self.advance();
                return self.consume_if_statement();
            }
            _=> {
                self.expression_statement()
            }
        }
    }

    fn consume_if_statement(&mut self) -> Result<Stmt, ParseError> {
        // Parse main if condition
        let condition = self.expression()?;
        
        // Parse if body
        self.consume(TokenKind::LeftBrace, "Expected '{' after if condition")?;
        let mut then_branch = Vec::new();
        
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            then_branch.push(self.statement()?);
        }
        
        self.consume(TokenKind::RightBrace, "Expected '}' after if body")?;
        
        // Parse elif branches
        let mut elif_branches: Vec<(Expr, Vec<Stmt>)> = Vec::new();
        
        while self.check(TokenKind::Elif) {
            self.advance(); // consume 'elif'
            
            let elif_condition = self.expression()?;
            
            self.consume(TokenKind::LeftBrace, "Expected '{' after elif condition")?;
            let mut elif_body = Vec::new();
            
            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                elif_body.push(self.statement()?);
            }
            
            self.consume(TokenKind::RightBrace, "Expected '}' after elif body")?;
            
            elif_branches.push((elif_condition, elif_body));
        }
        
        // Parse else branch (optional)
        let mut else_branch: Option<Vec<Stmt>> = None;
        
        if self.check(TokenKind::Else) {
            self.advance(); // consume 'else'
            
            self.consume(TokenKind::LeftBrace, "Expected '{' after else")?;
            let mut else_body = Vec::new();
            
            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                else_body.push(self.statement()?);
            }
            
            self.consume(TokenKind::RightBrace, "Expected '}' after else body")?;
            
            else_branch = Some(else_body);
        }
        
        Ok(Stmt::If {
            condition,
            then_branch,
            elif_branches,
            else_branch,
        })
    }

    fn consume_label(&mut self) -> Result<Stmt, ParseError> {
        let mut label: Vec<(String, bool, Vec<String>, Vec<String>, Vec<Stmt>)> = Vec::new();

        let mut callable: bool;

        if self.check(TokenKind::At) {
            callable = false;  // Control flow label (has @)
        } else {
            callable = true;   // Callable label (no @)
        }

        if callable {
            // Get label name
            let name = self.consume_identifier("Expected label name")?;
            
            // Parse parameters
            self.consume(TokenKind::LeftParen, "Expected '(' after label name")?;
            
            let mut params: Vec<String> = Vec::new();      // External parameter names
            let mut internal_names: Vec<String> = Vec::new();  // Internal variable names
            
            while !self.check(TokenKind::RightParen) {
                let external_param = self.consume_identifier("Expected parameter name")?;
                self.consume(TokenKind::Equal, "Expected '=' in parameter mapping")?;
                let internal_name = self.consume_identifier("Expected internal variable name")?;
                
                params.push(external_param);           // Add to params
                internal_names.push(internal_name);    // Add to internal names
                
                if !self.matches(&[TokenKind::Comma]) {
                    break;
                }
            }
            
            self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
            
            // Parse body
            self.consume(TokenKind::LeftBrace, "Expected '{' before label body")?;
            let mut body: Vec<Stmt> = Vec::new();
            
            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                body.push(self.statement()?);
            }
            
            self.consume(TokenKind::RightBrace, "Expected '}' after label body")?;
            
            label.push((name, callable, params, internal_names, body));
            
        } else {
            // Control flow label code...
            let name = self.consume_identifier("Expected label name")?;
            self.consume(TokenKind::LeftBrace, "Expected '{' before label body")?;
            let mut body: Vec<Stmt> = Vec::new();
            
            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                body.push(self.statement()?);
            }
            
            self.consume(TokenKind::RightBrace, "Expected '}' after label body")?;
            
            label.push((name, callable, vec![], vec![], body));
        }

        Ok(Stmt::Label { _label_: label })
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if self.check(TokenKind::Identifier) {
            let token = self.advance();
            Ok(token.lexeme.clone())
        } else {
            Err(self.error(format!("Expected Identifier at line {}. {}", self.peek().line, message).as_str()))
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        Ok(Stmt::Expression(expr))
    }

    // =========================================================
    // Expressions (precedence climbing)
    // =========================================================

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_and()?;

        while self.matches(&[TokenKind::Or]) {
            let operator = self.previous().clone();
            let right = self.logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenKind::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenKind::Star, TokenKind::Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().kind {
            TokenKind::Bang | TokenKind::Minus => {
                let operator = self.advance();
                let right = self.unary()?;
                Ok(Expr::Unary {
                    operator,
                    right: Box::new(right),
                })
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().kind {
            TokenKind::False => {
                self.advance();
                Ok(Expr::_Literal_(Literal::Bool(false)))
            }

            TokenKind::True => {
                self.advance();
                Ok(Expr::_Literal_(Literal::Bool(true)))
            }

            TokenKind::Nil => {
                self.advance();
                Ok(Expr::_Literal_(Literal::Nil))
            }

            TokenKind::Number => {
                let token = self.advance();

                // Extract number literal from token
                let literal = if let Some(crate::lexer::Literal::Number(num_lit)) = &token.literal {
                    match num_lit {
                        crate::lexer::tokens::NumberLit::Int(i) => Literal::Int(*i),
                        crate::lexer::tokens::NumberLit::Float(f) => Literal::Float(*f),
                        crate::lexer::tokens::NumberLit::BigIntString(s) => {
                            Literal::BigInt(s.clone())
                        }
                    }
                } else {
                    // Fallback: parse from lexeme as f64 if literal is missing
                    let value = token
                        .lexeme
                        .parse::<f64>()
                        .map_err(|_| self.error("Invalid number literal"))?;
                    Literal::Float(value)
                };

                Ok(Expr::_Literal_(literal))
            }

            TokenKind::String => {
                let token = self.advance();

                // Extract string literal from token
                let value = if let Some(crate::lexer::Literal::String(s)) = &token.literal {
                    s.clone()
                } else {
                    // Fallback: use lexeme (includes quotes)
                    token.lexeme.clone()
                };

                Ok(Expr::_Literal_(Literal::String(value)))
            }

            TokenKind::Char => {
                let token = self.advance();

                // Extract char literal from token
                let value = if let Some(crate::lexer::Literal::Char(c)) = &token.literal {
                    *c
                } else {
                    return Err(self.error("Invalid character literal"));
                };

                Ok(Expr::_Literal_(Literal::Char(value)))
            }

            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }

            TokenKind::Log => {
                self.advance();
                let expr = self.expression()?;
                Ok(Expr::Log(Box::new(expr)))
            }

            TokenKind::Identifier => self.scan_identifier(),

            _ => Err(self.error("Expect expression")),
        }
    }

    //==========================================================
    // Grammer
    fn scan_identifier(&mut self) -> Result<Expr, ParseError> {
        let identifier: String = self.peek().lexeme.clone();
        self.advance();

        match self.peek().kind {
            /*
                Function Call
                call(src=[34, 34])
                call(src["Hello From", "Danishk"])
            */
            TokenKind::LeftParen => {
                let mut args_map: Vec<(String, Expr)> = Vec::new();
                self.advance();
                while !self.check(TokenKind::RightParen) {
                    let name: String = self
                        .consume(
                            TokenKind::Identifier,
                            "Expected 'Identifier' for mapping args to parameters",
                        )
                        .unwrap()
                        .lexeme;
                    self.consume(
                        TokenKind::Equal,
                        "Expected '=' to differentiate name and expression",
                    )
                    .unwrap();
                    let value: Expr = self.expression()?;
                    args_map.push((name, value));

                    if self.check(TokenKind::Comma) {
                        self.advance(); // ',' skip
                    } else {
                        break;
                    }
                }
                self.consume(
                    TokenKind::RightParen,
                    "Expected ')' to enclose function call",
                ).unwrap();
                Ok(Expr::FunctionCall { function:identifier, args: args_map })
            }

            TokenKind::Equal => {
                self.advance();
                let value: Expr = self.expression()?;
                Ok(Expr::AllocateVariable {
                    name: identifier,
                    val: Box::new(value),
                })
            }

            _ => Ok(Expr::Variable { name: identifier }),
        }
    }
    //==========================================================

    // =========================================================
    // Cursor utilities
    // =========================================================

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.peek().kind == kind
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        for &k in kinds {
            if self.check(k) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    // =========================================================
    // Error handling
    // =========================================================

    fn error(&mut self, message: &str) -> ParseError {
        let err = ParseError {
            token: self.peek().clone(),
            message: message.to_string(),
        };
        self.errors.push(err.clone());
        err
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::If | TokenKind::Return => return,
                _ => {
                    self.advance();
                }
            };
        }
    }
}
