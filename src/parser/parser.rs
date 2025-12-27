use std::collections::HashMap;
use std::fmt::Arguments;

use crate::lexer::{Token, TokenKind};
use crate::parser::ast::{Expr, Literal, Stmt, StructMethod};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
    macro_map: HashMap<String, (Vec<String>, Vec<Stmt>)>,
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
            macro_map: HashMap::new(),
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
                let identifier: String = self
                    .consume(TokenKind::Identifier, "Expected 'Identifier'")
                    .unwrap()
                    .lexeme;
                Ok(Stmt::SmartLock {
                    variable: identifier,
                })
            }

            TokenKind::VUnlock => {
                self.advance();
                let identifier: String = self
                    .consume(TokenKind::Identifier, "Expected 'Identifier'")
                    .unwrap()
                    .lexeme;
                Ok(Stmt::SmartUnlock {
                    variable: identifier,
                })
            }

            TokenKind::VKill => {
                self.advance();
                let identifier: String = self
                    .consume(TokenKind::Identifier, "Expected 'Identifier'")
                    .unwrap()
                    .lexeme;
                Ok(Stmt::SmartKill {
                    variable: identifier,
                })
            }

            TokenKind::VRevive => {
                self.advance();
                let identifier: String = self
                    .consume(TokenKind::Identifier, "Expected 'Identifier'")
                    .unwrap()
                    .lexeme;
                Ok(Stmt::SmartRevive {
                    variable: identifier,
                })
            }

            TokenKind::VConst => {
                self.advance();
                let identifier: String = self
                    .consume(TokenKind::Identifier, "Expected 'Identifier'")
                    .unwrap()
                    .lexeme;
                Ok(Stmt::SmartConst {
                    variable: identifier,
                })
            }
            TokenKind::Label => {
                self.advance();
                self.consume_label()
            }
            TokenKind::If => {
                self.advance();
                self.consume_if_statement()
            }
            TokenKind::Jump => {
                self.advance();
                let _where_: String = self
                    .consume_identifier("Expected 'identifier' after Jump")
                    .unwrap();
                Ok(Stmt::Jump { jump: _where_ })
            }
            TokenKind::Pass => {
                self.advance();
                Ok(Stmt::Pass)
            }

            TokenKind::For => self.for_loop(),

            TokenKind::Do => self.do_while_loop(),

            TokenKind::While => self.while_loop(),

            TokenKind::Visible => self.def_visible_block(),

            TokenKind::DEFINE => {
                self.advance();
                self.define_macro()
            }
            TokenKind::IFNDEF => {
                self.advance();
                self.ifndef_macro()
            }
            TokenKind::UNDEF => {
                self.advance();
                self.undef_macro()
            }

            TokenKind::Struct => {
                self.advance();
                self.parse_struct_definition()
            }

            _ => self.expression_statement(),
        }
    }

    fn parse_struct_definition(&mut self) -> Result<Stmt, ParseError> {
        let struct_name = self.consume_identifier("Expected struct name")?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after struct name")?;

        let mut methods = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Parse method definition - allow 'constructor' keyword as method name
            let method_name = if self.check(TokenKind::Constructor) {
                self.advance();
                "constructor".to_string()
            } else {
                self.consume_identifier("Expected method name")?
            };

            self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;

            let mut params = Vec::new();
            while !self.check(TokenKind::RightParen) {
                // Allow 'self' keyword as parameter name
                let param_name = if self.check(TokenKind::Self_) {
                    self.advance();
                    "self".to_string()
                } else {
                    self.consume_identifier("Expected parameter name")?
                };
                params.push(param_name);

                if !self.matches(&[TokenKind::Comma]) {
                    break;
                }
            }

            self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
            self.consume(TokenKind::LeftBrace, "Expected '{' before method body")?;

            let mut body = Vec::new();
            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                body.push(self.statement()?);
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after method body")?;

            methods.push(StructMethod {
                name: method_name,
                params,
                body,
            });
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after struct body")?;

        Ok(Stmt::StructDef {
            name: struct_name,
            methods,
        })
    }

    fn undef_macro(&mut self) -> Result<Stmt, ParseError> {
        let macro_name = self.consume_identifier("Expected Macro name after _undef_ keyword")?;
        if self.macro_map.contains_key(&macro_name) {
            self.macro_map.remove(&macro_name);
        } else {
            return Err(self.error(
                format!("_undef_ {}, but macro not define in this scope", macro_name).as_str(),
            ));
        }
        Ok(Stmt::Pass)
    }

    fn ifndef_macro(&mut self) -> Result<Stmt, ParseError> {
        let macro_name = self.consume_identifier("Expected Macro Name")?;

        if !self.macro_map.contains_key(&macro_name) {
            // Macro NOT defined - define macros inside the block
            while !self.check(TokenKind::ENDIF) && !self.is_at_end() {
                self.consume(TokenKind::DEFINE, "Expected '_macro_' keyword")?;
                self.define_macro()?;
            }
            self.consume(TokenKind::ENDIF, "Expected 'ENDIF' to close 'IFNDEF' block")?;
            Ok(Stmt::Pass)
        } else {
            // Macro IS defined - skip everything until ENDIF
            while !self.check(TokenKind::ENDIF) && !self.is_at_end() {
                self.skip_macro()?; // Skip each macro definition
            }
            self.consume(TokenKind::ENDIF, "Expected 'ENDIF' to close 'IFNDEF' block")?;

            Ok(Stmt::Pass) // Added return value
        }
    }

    fn skip_macro(&mut self) -> Result<Stmt, ParseError> {
        self.consume_identifier("Expected Macro Name")?;

        self.consume(TokenKind::LeftParen, "Expected '(' for macro parameters")?;
        while !self.check(TokenKind::RightParen) {
            self.consume_identifier("Expected 'identifier' as macro parameters")?;

            // Optional: handle comma separation
            if self.check(TokenKind::Comma) {
                self.advance();
            }
        }
        self.consume(
            TokenKind::RightParen,
            "Expected ')' to enclose macro parameters",
        )?;
        self.consume(TokenKind::LeftBracket, "Expected '[' to enclose macro body")?;

        while !self.check(TokenKind::RightBracket) {
            self.statement()?;
        }
        self.consume(
            TokenKind::RightBracket,
            "Expected ']' to enclose macro body",
        )?; // Fixed: was RightParen

        Ok(Stmt::Pass)
    }

    fn define_macro(&mut self) -> Result<Stmt, ParseError> {
        let macro_name = self.consume_identifier("Expected Macro Name")?;
        let mut macro_param: Vec<String> = Vec::new();

        self.consume(TokenKind::LeftParen, "Expected '(' for macro parameters")?;
        while !self.check(TokenKind::RightParen) {
            macro_param.push(self.consume_identifier("Expected 'identifier' as macro parameters")?);

            // Optional: handle comma separation
            if self.check(TokenKind::Comma) {
                self.advance();
            }
        }
        self.consume(
            TokenKind::RightParen,
            "Expected ')' to enclose macro parameters",
        )?;

        self.consume(TokenKind::LeftBracket, "Expected '[' to enclose macro body")?;
        let mut macro_body: Vec<Stmt> = Vec::new();
        while !self.check(TokenKind::RightBracket) {
            macro_body.push(self.statement()?);
        }
        self.consume(
            TokenKind::RightBracket,
            "Expected ']' to enclose macro body",
        )?; // Fixed: was RightParen

        let _macro_: (Vec<String>, Vec<Stmt>) = (macro_param, macro_body);
        self.macro_map.insert(macro_name, _macro_);

        Ok(Stmt::Pass)
    }

    fn def_visible_block(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let name: String = self.consume_identifier("Expected visible block name")?;
        self.consume(
            TokenKind::LeftParen,
            "Expected '(' to enclose visible block",
        )?;

        let mut block: Vec<(String, Expr)> = Vec::new();

        while !self.check(TokenKind::RightParen) {
            let identifier = self.consume_identifier("Expected 'Identifier' as variable name")?;
            self.consume(TokenKind::Equal, "Expected '=' after variable name")?;
            let value: Expr = self.expression()?;
            block.push((identifier, value));

            if self.matches(&[TokenKind::Comma]) {
                continue;
            }
            break;
        }
        self.consume(
            TokenKind::RightParen,
            "Expected '(' to enclose visible block",
        )?;
        Ok(Stmt::Visible {
            _name_: name,
            _block_: block,
        })
    }

    fn while_loop(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'while'

        let condition = self.expression()?;
        self.consume(TokenKind::LeftBrace, "Expected '{' after while condition")?;

        let mut body = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.statement()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after while body")?;
        Ok(Stmt::While { condition, body })
    }

    fn do_while_loop(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'do'

        self.consume(TokenKind::LeftBrace, "Expected '{' after 'do'")?;

        let mut body = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.statement()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after do body")?;
        self.consume(TokenKind::While, "Expected 'while' after do-while body")?;
        let condition = self.expression()?;

        Ok(Stmt::DoWhile { body, condition })
    }

    fn for_loop(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'for'

        let iterator = self.consume_identifier("Expected iterator variable in for loop")?;
        self.consume(TokenKind::In, "Expected 'in' keyword in for loop")?;
        let iterable = self.expression()?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after for loop header")?;

        let mut body = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.statement()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after for loop body")?;

        Ok(Stmt::For {
            iterator,
            iterable,
            body,
        })
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
        let mut label: Vec<(
            String,
            bool,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<Stmt>,
        )> = Vec::new();

        let callable: bool;

        if self.check(TokenKind::At) {
            callable = false; // Control flow label (has @)
        } else {
            callable = true; // Callable label (no @)
        }

        if callable {
            // Get label name
            let mut visit: Vec<String> = Vec::new();
            self.consume(TokenKind::Visit, "Expected 'visit' keyword after label")?;
            self.consume(
                TokenKind::LeftBracket,
                "Expected '[' to eclose Left Barcket",
            )?;
            while !self.check(TokenKind::RightBracket) {
                visit.push(
                    self.consume_identifier("Expected 'identifier for visible block'")
                        .unwrap(),
                );
                if self.matches(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }
            self.consume(
                TokenKind::RightBracket,
                "Expected ']' to enclose function visit",
            )?;
            let name = self.consume_identifier("Expected label name")?;
            self.consume(TokenKind::LeftParen, "Expected '(' after label name")?;

            let mut params: Vec<String> = Vec::new(); // External parameter names
            let mut internal_names: Vec<String> = Vec::new(); // Internal variable names

            while !self.check(TokenKind::RightParen) {
                let external_param = self.consume_identifier("Expected parameter name")?;
                self.consume(TokenKind::Equal, "Expected '=' in parameter mapping")?;
                let internal_name = self.consume_identifier("Expected internal variable name")?;

                params.push(external_param); // Add to params
                internal_names.push(internal_name); // Add to internal names

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

            label.push((name, callable, visit, params, internal_names, body));
        } else {
            // Control flow label code...
            self.advance();
            let name = self.consume_identifier("Expected label name")?;
            self.consume(TokenKind::LeftBrace, "Expected '{' before label body")?;
            let mut body: Vec<Stmt> = Vec::new();

            while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                body.push(self.statement()?);
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after label body")?;

            label.push((name, callable, vec![], vec![], vec![], body));
        }

        Ok(Stmt::Label { _label_: label })
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if self.check(TokenKind::Identifier) {
            let token = self.advance();
            Ok(token.lexeme.clone())
        } else {
            Err(self.error(
                format!(
                    "Expected Identifier at line {}. {}",
                    self.peek().line,
                    message
                )
                .as_str(),
            ))
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
            TokenKind::ColonColon => {
                self.advance();
                self.consume(
                    TokenKind::LeftBracket,
                    "Expected '[' to consume dynamic array",
                )?;

                let start: String = self
                    .consume(
                        TokenKind::Number,
                        "Expected Starting Number to create dynamic Array",
                    )
                    .unwrap()
                    .lexeme;

                self.consume(TokenKind::Dot, "Missing a '.' in for loop")?;
                self.consume(TokenKind::Dot, "Missing another '.' in for loop")?;
                let end: String = self
                    .consume(
                        TokenKind::Number,
                        "Expected Starting Number to create dynamic Array",
                    )
                    .unwrap()
                    .lexeme;

                self.consume(
                    TokenKind::RightBracket,
                    "Expected ']' to consume dynamic array",
                )?;

                let _start_ = start.parse::<i128>().unwrap();
                let _end_ = end.parse::<i128>().unwrap();

                let mut values = Vec::new();

                if _start_ <= _end_ {
                    for i in _start_..=_end_ {
                        values.push(i);
                    }
                } else {
                    // descending range support
                    for i in (_end_..=_start_).rev() {
                        values.push(i);
                    }
                }

                Ok(Expr::Iterable { value: values })
            }

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

            TokenKind::Print => {
                self.advance();
                let expr = self.expression()?;
                Ok(Expr::Print(Box::new(expr)))
            }

            TokenKind::Hash => {
                self.advance();
                let macro_name = self
                    .consume_identifier("Expected 'Identifier as macro name'")
                    .unwrap();
                let mut args: Vec<Expr> = Vec::new();

                if self.macro_map.contains_key(&macro_name) {
                    self.consume(TokenKind::LeftParen, "Expected '(' to capture macro args")?;
                    while !self.check(TokenKind::RightParen) {
                        args.push(self.expression()?);
                        if self.matches(&[TokenKind::Comma]) {
                            continue;
                        }
                        break;
                    }
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' to enclose macro arguments",
                    )?;

                    let macro_body = match self.macro_map.get(&macro_name) {
                        Some(body) => body,
                        None => {
                            return Err(self.error(&format!("undefined macro {}", macro_name)));
                        }
                    };
                    let (macro_params, macro_stmts) = macro_body;

                    let mut variables: Vec<Expr> = Vec::new();
                    for (_param_, _arg_) in macro_params.iter().zip(args) {
                        variables.push(Expr::AllocateVariable {
                            name: _param_.clone(),
                            val: Box::new(_arg_),
                        });
                    }

                    Ok(Expr::MacroCall {
                        var: variables,
                        body: macro_stmts.clone(),
                    })
                } else {
                    Err(self.error(&format!(
                        "Macro {} is not define anywhere in code",
                        macro_name
                    )))
                }
            }

            TokenKind::Self_ => {
                self.advance(); // consume 'self'

                // base expr is the 'self' variable
                let mut expr = Expr::Variable {
                    name: "self".to_string(),
                };

                // support: self.field, self.field = value, self.method(...)
                while self.matches(&[TokenKind::Dot]) {
                    let member = self.consume_identifier("Expected member name after '.'")?;

                    // method call: self.method(...)
                    if self.check(TokenKind::LeftParen) {
                        self.advance(); // consume '('

                        let mut args = Vec::new();
                        while !self.check(TokenKind::RightParen) {
                            args.push(self.expression()?);

                            if !self.matches(&[TokenKind::Comma]) {
                                break;
                            }
                        }

                        self.consume(TokenKind::RightParen, "Expected ')' after method arguments")?;

                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: member,
                            args,
                        };
                        continue;
                    }

                    // assignment: self.field = value
                    if self.check(TokenKind::Equal) {
                        self.advance(); // consume '='
                        let value = self.expression()?;

                        return Ok(Expr::MemberAssign {
                            object: Box::new(expr),
                            member,
                            value: Box::new(value),
                        });
                    }

                    // access: self.field
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                    };
                }

                Ok(expr)
            }

            TokenKind::Identifier => self.scan_identifier(),

            _ => Err(self.error("Expect expression")),
        }
    }

    //==========================================================
    // Grammer
    //==========================================================

    fn scan_identifier(&mut self) -> Result<Expr, ParseError> {
        let identifier: String = self.peek().lexeme.clone();
        self.advance(); // consume the identifier

        match self.peek().kind {
            // ---------------------------------------------------
            // Struct / Static call: student::new(...)
            // ---------------------------------------------------
            TokenKind::ColonColon => {
                self.advance(); // consume '::'

                // IMPORTANT FIX:
                // after '::' allow either Identifier OR keyword 'new'
                let method_name: String = if self.check(TokenKind::Identifier) {
                    self.advance().lexeme.clone()
                } else if self.check(TokenKind::New) {
                    // if your token is named New_ or NEW, change it accordingly
                    self.advance(); // consume 'new'
                    "new".to_string()
                } else {
                    return Err(self.error(
                        format!(
                            "Expected Identifier at line {}. Expected method name after '::'",
                            self.peek().line
                        )
                        .as_str(),
                    ));
                };

                self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;

                let mut args: Vec<Expr> = Vec::new();
                while !self.check(TokenKind::RightParen) {
                    args.push(self.expression()?);

                    if !self.matches(&[TokenKind::Comma]) {
                        break;
                    }
                }

                self.consume(TokenKind::RightParen, "Expected ')' after arguments")?;

                Ok(Expr::StructInstantiation {
                    struct_name: identifier,
                    method_name,
                    args,
                })
            }

            // ---------------------------------------------------
            // Member access / method call / member assign:
            // obj.member, obj.method(...), obj.member = value
            // ---------------------------------------------------
            TokenKind::Dot => {
                let mut expr = Expr::Variable { name: identifier };

                while self.matches(&[TokenKind::Dot]) {
                    let member = self.consume_identifier("Expected member name after '.'")?;

                    if self.check(TokenKind::LeftParen) {
                        // Method call: obj.method(...)
                        self.advance(); // consume '('

                        let mut args = Vec::new();
                        while !self.check(TokenKind::RightParen) {
                            args.push(self.expression()?);

                            if !self.matches(&[TokenKind::Comma]) {
                                break;
                            }
                        }

                        self.consume(TokenKind::RightParen, "Expected ')' after method arguments")?;

                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: member,
                            args,
                        };
                    } else if self.check(TokenKind::Equal) {
                        // Member assignment: obj.member = value
                        self.advance(); // consume '='
                        let value = self.expression()?;

                        return Ok(Expr::MemberAssign {
                            object: Box::new(expr),
                            member,
                            value: Box::new(value),
                        });
                    } else {
                        // Member access: obj.member
                        expr = Expr::MemberAccess {
                            object: Box::new(expr),
                            member,
                        };
                    }
                }

                Ok(expr)
            }

            // ---------------------------------------------------
            // Function call with named params:
            // foo(a=1, b=2)
            // ---------------------------------------------------
            TokenKind::LeftParen => {
                let mut args_map: Vec<(String, Expr)> = Vec::new();
                self.advance(); // consume '('

                while !self.check(TokenKind::RightParen) {
                    let name: String = self
                        .consume(
                            TokenKind::Identifier,
                            "Expected 'Identifier' for mapping args to parameters",
                        )?
                        .lexeme;

                    self.consume(
                        TokenKind::Equal,
                        "Expected '=' to differentiate name and expression",
                    )?;

                    let value: Expr = self.expression()?;
                    args_map.push((name, value));

                    if self.check(TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                self.consume(TokenKind::RightParen, "Expected ')' to enclose function call")?;

                Ok(Expr::FunctionCall {
                    function: identifier,
                    args: args_map,
                })
            }

            // ---------------------------------------------------
            // Variable assignment: x = expr
            // ---------------------------------------------------
            TokenKind::Equal => {
                self.advance(); // consume '='
                let value: Expr = self.expression()?;

                Ok(Expr::AllocateVariable {
                    name: identifier,
                    val: Box::new(value),
                })
            }

            // ---------------------------------------------------
            // Just a variable reference
            // ---------------------------------------------------
            _ => Ok(Expr::Variable { name: identifier }),
        }
    }
//==========================================================

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
