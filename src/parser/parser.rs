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

    /// Parse a full program (list of statements)
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
    // Declarations & Statements (STRUCTURE ONLY)
    // =========================================================

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        // later: var / fun / class
        self.statement()
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        // later: if / while / for / return / block
        self.expression_statement()
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        Ok(Stmt::Expression(expr))
    }

    // =========================================================
    // Expression grammar (Crafting Interpreters)
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

        while self.matches(&[
            TokenKind::BangEqual,
            TokenKind::EqualEqual,
        ]) {
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

        while self.matches(&[
            TokenKind::Plus,
            TokenKind::Minus,
        ]) {
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

        while self.matches(&[
            TokenKind::Star,
            TokenKind::Slash,
        ]) {
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
        if self.matches(&[
            TokenKind::Bang,
            TokenKind::Minus,
        ]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[TokenKind::False]) {
            return Ok(Expr::_Literal_(Literal::Boolean(false)));
        }

        if self.matches(&[TokenKind::True]) {
            return Ok(Expr::_Literal_(Literal::Boolean(true)));
        }

        if self.matches(&[TokenKind::Nil]) {
            return Ok(Expr::_Literal_(Literal::Nil));
        }

        if self.matches(&[TokenKind::Number]) {
            let token = self.previous().clone();
            let literal = token.lexeme;
            return Ok(Expr::_Literal_(Literal::Number(literal.parse::<f64>().unwrap())))
        }

        if self.matches(&[TokenKind::String]) {
            let token = self.previous().clone();
            let literal = token.lexeme;
            return Ok(Expr::_Literal_(Literal::String(literal)));
        }

        if self.matches(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.matches(&[TokenKind::Log]) {
            let expr = self.expression()?;
            return Ok(Expr::Log(Box::new(expr)))
        }

        Err(self.error("Expect expression"))
    }

    // =========================================================
    // Literal conversion (IMPORTANT FIX)
    // =========================================================



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
                TokenKind::If
                | TokenKind::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
