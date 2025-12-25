use std::collections::HashMap;

use crate::lexer::{Token, TokenKind};

use super::ast::*;

/// JS: ParseError(token, message) :contentReference[oaicite:9]{index=9}
#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl ParseError {
    pub fn new(token: Token, message: impl Into<String>) -> Self {
        Self {
            token,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct MacroDef {
    params: Vec<Token>,
    body: Vec<Stmt>,
}

/// Main parser (merges parser-base + mixins) :contentReference[oaicite:10]{index=10}
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub has_error: bool,
    macro_map: HashMap<String, MacroDef>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            has_error: false,
            macro_map: HashMap::new(),
        }
    }

    /// Entry point: parse whole program :contentReference[oaicite:11]{index=11}
    pub fn parse(&mut self) -> Program {
        let mut program = Program {
            visible_soft: None,
            visible_hard: None,
            labels: vec![],
            statements: vec![],
        };

        while !self.is_at_end() {
            match self.program_element() {
                Ok(Some(elem)) => match elem {
                    TopLevel::Visible(vb) => match vb.kind {
                        VisibleKind::VisibleSoft => program.visible_soft = Some(vb),
                        VisibleKind::VisibleHard => program.visible_hard = Some(vb),
                    },
                    TopLevel::Label(ld) => program.labels.push(ld),
                    TopLevel::Struct(s) => {
                        // Add struct as a statement
                        program.statements.push(Stmt::StructDecl {
                            name: s.name,
                            constructor: s.constructor,
                            methods: s.methods,
                        });
                    }
                    TopLevel::Stmt(s) => program.statements.push(s),
                },
                Ok(None) => {}
                Err(e) => {
                    self.report_error(&e);
                    self.synchronize();
                }
            }
        }

        program
    }

    // =========================================================
    // Helpers (from parser-helpers.js) :contentReference[oaicite:14]{index=14}
    // =========================================================

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for &k in kinds {
            if self.check(k) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind == kind
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(self.peek().clone(), msg))
        }
    }

    fn error(&mut self, token: Token, msg: &str) -> ParseError {
        self.has_error = true;
        ParseError::new(token, msg)
    }

    fn report_error(&self, err: &ParseError) {
        let token = &err.token;
        let location = if token.kind == TokenKind::Eof {
            "at end".to_string()
        } else {
            format!("at '{}'", token.lexeme)
        };
        eprintln!(
            "[Parser Error] line {} {}: {}",
            token.line, location, err.message
        );
    }

    fn is_statement_end(&self) -> bool {
        self.check(TokenKind::RightBracket) || self.check(TokenKind::RightBrace) || self.is_at_end()
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::Label | TokenKind::If | TokenKind::Struct => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    // =========================================================
    // Program-level parsing (parser-program.js) :contentReference[oaicite:15]{index=15}
    // =========================================================

    fn program_element(&mut self) -> Result<Option<TopLevel>, ParseError> {
        // Visible blocks start with '*'
        if self.match_any(&[TokenKind::Star]) {
            let vb = self.visible_block()?;
            return Ok(Some(TopLevel::Visible(vb)));
        }

        // Label decl
        if self.match_any(&[TokenKind::Label]) {
            if self.check(TokenKind::At) {
                let l = self.control_flow_label()?;
                return Ok(Some(TopLevel::Label(l)));
            } else {
                let l = self.callable_label()?;
                return Ok(Some(TopLevel::Label(l)));
            }
        }

        // Preprocessor
        if self.match_any(&[TokenKind::Hash]) {
            if self.match_any(&[TokenKind::IfDef]) {
                let s = self.if_define_stmt()?;
                return Ok(Some(TopLevel::Stmt(s)));
            }
            if self.match_any(&[TokenKind::IfNDef]) {
                let s = self.if_not_define_stmt()?;
                return Ok(Some(TopLevel::Stmt(s)));
            }
            if self.match_any(&[TokenKind::UnDef]) {
                let s = self.undefine_stmt()?;
                return Ok(Some(TopLevel::Stmt(s)));
            }
            if self.match_any(&[TokenKind::DefineMacro]) {
                self.handle_define_macro()?;
                return Ok(None);
            }
        }

        // Struct
        if self.check(TokenKind::Struct) {
            self.advance();
            let st = self.struct_declaration()?;
            return Ok(Some(TopLevel::Struct(st))); // New variant
        }

        // Regular statement
        let s = self.statement()?;
        Ok(Some(TopLevel::Stmt(s)))
    }

    fn visible_block(&mut self) -> Result<VisibleBlock, ParseError> {
        self.consume(TokenKind::LeftBracket, "Expected '[' after '*'")?;

        // JS reads blockType from next token lexeme (visible_soft/visible_hard) :contentReference[oaicite:16]{index=16}
        let block_type_tok = self.peek().clone();
        let block_lex = block_type_tok.lexeme.as_str();

        let kind = match block_lex {
            "visible_soft" => VisibleKind::VisibleSoft,
            "visible_hard" => VisibleKind::VisibleHard,
            _ => {
                return Err(self.error(
                    block_type_tok,
                    "expected visible_hard or visible_soft to ensure a static block behavior",
                ));
            }
        };
        self.advance(); // consume visible_* token

        self.consume(
            TokenKind::LeftParen,
            "Expected '(' to enclose visible block",
        )?;
        let parameter = self.consume(
            TokenKind::Identifier,
            "Expected 'IDENTIFIER' make static block as visible",
        )?;
        self.consume(
            TokenKind::RightParen,
            "Expected ')' to enclose visible block",
        )?;

        self.consume(
            TokenKind::RightBracket,
            "Expected ']' after visibility type",
        )?;
        self.consume(
            TokenKind::LeftBracket,
            "Expected '[' to start visible block body",
        )?;

        let mut declarations = vec![];
        while !self.check(TokenKind::RightBracket) && !self.is_at_end() {
            declarations.push(self.expression()?);
        }
        self.consume(
            TokenKind::RightBracket,
            "Expected ']' to close visible block",
        )?;

        Ok(VisibleBlock {
            kind,
            parameter,
            declarations,
        })
    }

    fn struct_declaration(&mut self) -> Result<StructDecl, ParseError> {
        let name = self.consume(
            TokenKind::Identifier,
            "Expected 'structure name' after struct keyword",
        )?;
        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' after structure to enclose structure properties",
        )?;
        let constructor = self.constructor_body()?;

        let mut methods = vec![];
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.struct_method()?);
        }

        self.consume(
            TokenKind::RightBrace,
            "Expected '}' after structure, to enclose structure body",
        )?;
        Ok(StructDecl {
            name,
            constructor,
            methods,
        })
    }

    fn constructor_body(&mut self) -> Result<ConstructorDecl, ParseError> {
        self.consume(
            TokenKind::Constructor,
            "Expected At least one constructor to define variables in structure's Global Scope",
        )?;
        let name = self.consume(
            TokenKind::Identifier,
            "Expected 'Identifier' as function name after constructor",
        )?;

        self.consume(
            TokenKind::LeftParen,
            "Expected '(' to enclose constructor function's parameters",
        )?;
        let mut self_flag = false;
        let mut params: Vec<ConstructorParam> = vec![];

        while !self.check(TokenKind::RightParen) && !self.is_at_end() {
            if self.check(TokenKind::SelfKw) {
                if self_flag {
                    return Err(self.error(
                        self.peek().clone(),
                        "Only one 'self' parameter allowed in constructor",
                    ));
                }
                let t = self.advance();
                params.push(ConstructorParam::SelfParam(t)); // <-- This line constructs it
                self_flag = true;
            } else if self.match_any(&[TokenKind::Define]) {
                let pn = self.consume(
                    TokenKind::Identifier,
                    "Expected parameter name after 'define'",
                )?;
                params.push(ConstructorParam::DefinedParam(pn));
            } else if self.check(TokenKind::Identifier) {
                let t = self.advance();
                params.push(ConstructorParam::Ident(t));
            } else {
                return Err(self.error(
                    self.peek().clone(),
                    "Expected parameter name, 'define', or 'self' in constructor parameters",
                ));
            }

            if !self.check(TokenKind::RightParen) {
                if !self.match_any(&[TokenKind::Comma]) {
                    return Err(self.error(
                        self.peek().clone(),
                        "Expected ',' between parameters or ')' to close parameters",
                    ));
                }
            }
        }

        self.consume(
            TokenKind::RightParen,
            "Expected ')' to close constructor parameters",
        )?;
        self.consume(
            TokenKind::LeftBracket,
            "Expected '[' to enclose structure's constructor body",
        )?;
        let body = self.block_statements()?; // consumes RightBracket

        Ok(ConstructorDecl {
            name,
            parameters: params,
            body,
        })
    }

    fn struct_method(&mut self) -> Result<StructMethod, ParseError> {
        let access_modifier = if self.match_any(&[TokenKind::Private]) {
            AccessModifier::Private
        } else if self.match_any(&[TokenKind::Public]) {
            AccessModifier::Public
        } else {
            return Err(self.error(
                self.previous().clone(),
                "Expected 'public/private' access modifiers in structure method",
            ));
        };

        let name = self.consume(TokenKind::Identifier, "Expected method name")?;

        self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;
        let mut parameters = vec![];
        if !self.check(TokenKind::RightParen) {
            loop {
                parameters.push(self.expression()?);
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }
        }
        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        let visibility = if self.match_any(&[TokenKind::Ampersand]) {
            self.consume(TokenKind::LeftBracket, "Expected '[' after '&'")?;
            self.consume(
                TokenKind::Visibility,
                "Expected 'visibility' to ensure its accessibility",
            )?;
            self.consume(
                TokenKind::LeftParen,
                "Expected '(' to enclose method visibility area",
            )?;

            let mut vis = vec![];
            while !self.check(TokenKind::RightParen) && !self.is_at_end() {
                vis.push(self.consume(
                    TokenKind::Identifier,
                    "Expected 'Identifier' to represent visibility as name",
                )?);
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }

            self.consume(
                TokenKind::RightParen,
                "Expected ')' to enclose method visibility area",
            )?;
            self.consume(TokenKind::RightBracket, "Expected ']' after visibility")?;
            Some(vis)
        } else {
            None
        };

        self.consume(TokenKind::LeftBracket, "Expected '[' to start method body")?;
        let body = self.block_statements()?;

        Ok(StructMethod {
            access_modifier,
            name,
            parameters,
            visibility,
            body,
        })
    }

    fn handle_define_macro(&mut self) -> Result<(), ParseError> {
        let macro_name =
            self.consume(TokenKind::Identifier, "Expected Macro name as Identifier")?;
        let mut macro_params: Vec<Token> = vec![];

        if self.check(TokenKind::LeftParen) {
            self.consume(TokenKind::LeftParen, "Expected '(' to enclose macro params")?;
            while !self.is_at_end() && !self.check(TokenKind::RightParen) {
                macro_params.push(self.consume(
                    TokenKind::Identifier,
                    "Expected 'Identifier as macro parameters'",
                )?);
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }
            self.consume(
                TokenKind::RightParen,
                "Expected ')' to enclose macro parameters",
            )?;
        }

        let mut macro_body: Vec<Stmt> = vec![];
        self.consume(
            TokenKind::LeftBracket,
            "Expected '[' after to capture macro body",
        )?;
        while !self.is_at_end() && !self.check(TokenKind::RightBracket) {
            macro_body.push(self.statement()?);
        }
        self.consume(
            TokenKind::RightBracket,
            "Expected ']' to enclose macro body",
        )?;

        self.macro_map.insert(
            macro_name.lexeme.clone(),
            MacroDef {
                params: macro_params,
                body: macro_body,
            },
        );

        Ok(())
    }

    fn if_define_stmt(&mut self) -> Result<Stmt, ParseError> {
        let mac = self.consume(
            TokenKind::Identifier,
            "Expected identifier as a macro that is going to be checked as defined",
        )?;
        self.consume(
            TokenKind::LeftBracket,
            "Expected '[' to ensure #ifdef macro body",
        )?;
        let body = self.block_statements()?;
        Ok(Stmt::IfDef { name: mac, body })
    }

    fn if_not_define_stmt(&mut self) -> Result<Stmt, ParseError> {
        let mac = self.consume(
            TokenKind::Identifier,
            "Expected identifier as a macro that is going to be checked as if not defined",
        )?;
        self.consume(
            TokenKind::LeftBracket,
            "Expected '[' to ensure #ifdef macro body",
        )?;
        let body = self.block_statements()?;
        Ok(Stmt::IfNDef { name: mac, body })
    }

    fn undefine_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftBracket, "Expected [ to open undefine block")?;
        let mut macros = vec![];
        while !self.is_at_end() && !self.check(TokenKind::RightBracket) {
            macros.push(self.consume(
                TokenKind::Identifier,
                "Expected 'macro' name that is going to be undefined",
            )?);
            if self.match_any(&[TokenKind::Comma]) {
                continue;
            }
            break;
        }
        self.consume(
            TokenKind::RightBracket,
            "Expected ']' to enclose undefine macro body",
        )?;
        Ok(Stmt::Undef { macros })
    }

    // =========================================================
    // Statements (parser-statements.js) :contentReference[oaicite:17]{index=17}
    // =========================================================

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_any(&[TokenKind::LeftBracket]) {
            let stmts = self.block_statements()?;
            return Ok(Stmt::Block { statements: stmts });
        }

        if self.match_any(&[TokenKind::Label]) {
            if self.check(TokenKind::At) {
                let l = self.control_flow_label()?;
                return Ok(Stmt::Expression {
                    expression: Expr::Literal {
                        value: LiteralValue::Null,
                    },
                }); // unreachable in JS; labels handled top-level
            } else {
                let _l = self.callable_label()?;
                return Ok(Stmt::Expression {
                    expression: Expr::Literal {
                        value: LiteralValue::Null,
                    },
                });
            }
        }

        if self.match_any(&[TokenKind::Enum]) {
            return Ok(self.enum_declaration()?);
        }

        if self.match_any(&[TokenKind::Switch]) {
            return Ok(self.switch_statement()?);
        }

        if self.match_any(&[TokenKind::Jump]) {
            let id = self.consume(TokenKind::Identifier, "Expected 'Identifier at jump label'")?;
            return Ok(Stmt::Jump { target: id });
        }

        if self.match_any(&[TokenKind::Hash]) {
            if self.match_any(&[TokenKind::IfDef]) {
                return self.if_define_stmt();
            }
            if self.match_any(&[TokenKind::IfNDef]) {
                return self.if_not_define_stmt();
            }
            if self.match_any(&[TokenKind::UnDef]) {
                return self.undefine_stmt();
            }
            if self.match_any(&[TokenKind::DefineMacro]) {
                self.handle_define_macro()?;
                return Ok(Stmt::Block { statements: vec![] });
            }
        }

        if self.match_any(&[TokenKind::Eternal]) {
            let v = self.consume(
                TokenKind::Identifier,
                "Expected 'Identifier', while declaring a 'eternal' variable",
            )?;
            let value = if self.match_any(&[TokenKind::Equal]) {
                Some(self.expression()?)
            } else {
                None
            };
            return Ok(Stmt::Eternal { variable: v, value });
        }

        if self.match_any(&[TokenKind::_DEF_]) {
            self.consume(
                TokenKind::LeftParen,
                "Expected '(' to declare a default variable",
            )?;
            let variable = self.consume(
                TokenKind::Identifier,
                "Expected 'Identifier' as variable name",
            )?;
            self.consume(
                TokenKind::Comma,
                "Expected ',' to separate value from variable",
            )?;
            let default_v = self.expression()?;
            self.consume(TokenKind::RightParen, "Expected ')' to close default value")?;
            self.consume(
                TokenKind::Equal,
                "Expected '=' to capture default variable's mutable value",
            )?;
            let mut_v = self.expression()?;
            return Ok(Stmt::DefaultVariable {
                variable_name: variable,
                default_value: default_v,
                mutable_value: mut_v,
            });
        }

        if self.match_any(&[TokenKind::_TTV_]) {
            let var = self.consume(
                TokenKind::Identifier,
                "Expected 'Identifier' as variable name",
            )?;
            self.consume(TokenKind::Equal, "Expected '=' to capture variable's value")?;
            let value = self.expression()?;
            return Ok(Stmt::TTv {
                var_name: var,
                var_value: value,
            });
        }

        if self.match_any(&[TokenKind::_DELOCK_]) {
            let var = self.consume(
                TokenKind::Identifier,
                "Expected 'Identifier' as variable name",
            )?;
            self.consume(TokenKind::Equal, "Expected '=' to capture variable's value")?;
            let value = self.expression()?;
            return Ok(Stmt::Delock {
                var_name: var,
                var_value: value,
            });
        }

        if self.match_any(&[TokenKind::Rooted]) {
            let v = self.consume(
                TokenKind::Identifier,
                "Expected 'Identifier', while declaring a 'rooted' variable",
            )?;
            let value = if self.match_any(&[TokenKind::Equal]) {
                Some(self.expression()?)
            } else {
                None
            };
            return Ok(Stmt::Rooted { variable: v, value });
        }

        if self.match_any(&[TokenKind::If]) {
            return self.if_statement();
        }

        if self.match_any(&[TokenKind::Unlabel]) {
            let value = if self.is_statement_end() {
                None
            } else {
                Some(self.expression()?)
            };
            return Ok(Stmt::UnlabelStatement { value });
        }

        if self.match_any(&[TokenKind::Return]) {
            let expr = self.expression()?;
            return Ok(Stmt::Return { expression: expr });
        }

        // expression statement
        let expr = self.expression()?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.check(TokenKind::Identifier) {
            let checkpoint = self.current;
            let name = self.advance();
            if self.match_any(&[TokenKind::Equal]) {
                let init = self.expression()?;
                return Ok(Stmt::Var {
                    name,
                    initializer: init,
                });
            } else {
                self.current = checkpoint;
            }
        }
        self.statement()
    }

    fn block_statements(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = vec![];
        while !self.check(TokenKind::RightBracket) && !self.is_at_end() {
            let s = self.declaration()?;
            statements.push(s);
        }
        self.consume(TokenKind::RightBracket, "Expected ']' after block")?;
        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let cond = self.expression()?;
        self.consume(TokenKind::RightParen, "Expected ')' after condition")?;

        let then_branch = self.statement()?;
        let mut elifs: Vec<(Expr, Stmt)> = vec![];
        let mut else_branch: Option<Stmt> = None;

        while self.match_any(&[TokenKind::Elif]) {
            self.consume(TokenKind::LeftParen, "Expected '(' after 'elif'")?;
            let ec = self.expression()?;
            self.consume(TokenKind::RightParen, "Expected ')' after elif condition")?;
            let body = self.statement()?;
            elifs.push((ec, body));
        }

        if self.match_any(&[TokenKind::Else]) {
            else_branch = Some(self.statement()?);
        }

        // store as Block-like expression statement to keep AST compact:
        // you can make a dedicated Stmt::If if you prefer.
        Ok(Stmt::Expression {
            expression: Expr::Literal {
                value: LiteralValue::Null,
            },
        })
    }

    // =========================================================
    // Labels (parser-statements.js) :contentReference[oaicite:18]{index=18}
    // =========================================================

    fn callable_label(&mut self) -> Result<LabelDecl, ParseError> {
        let name = self.consume(TokenKind::Identifier, "Expected label name")?;

        self.consume(TokenKind::LeftParen, "Expected '(' after label name")?;
        let mut parameters: Vec<Expr> = vec![];
        if !self.check(TokenKind::RightParen) {
            loop {
                if self.match_any(&[TokenKind::Define]) {
                    let v = self.consume(
                        TokenKind::Identifier,
                        "Expected 'Identifier', while declaring a 'define' variable",
                    )?;
                    parameters.push(Expr::Define { variable: v });
                }
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }
        }
        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        let visibility = if self.match_any(&[TokenKind::Ampersand]) {
            self.consume(TokenKind::LeftBracket, "Expected '[' after '&'")?;
            self.consume(
                TokenKind::Visibility,
                "Expected 'visibility' to ensure its accessibility",
            )?;
            self.consume(
                TokenKind::LeftParen,
                "Expected '(' to enclose label based visibility area",
            )?;

            let mut vis = vec![];
            while !self.check(TokenKind::RightParen) && !self.is_at_end() {
                vis.push(self.consume(
                    TokenKind::Identifier,
                    "Expected 'Identifier' to represent visibility as name",
                )?);
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }

            self.consume(
                TokenKind::RightParen,
                "Expected ')' to enclose label based visibility area",
            )?;
            self.consume(TokenKind::RightBracket, "Expected ']' after visibility")?;
            Some(vis)
        } else {
            None
        };

        self.consume(
            TokenKind::LeftBracket,
            "Expected '[' to start callable label body",
        )?;
        let body = self.block_statements()?;

        Ok(LabelDecl::Callable(CallableLabel {
            name,
            parameters,
            visibility,
            body,
        }))
    }

    fn control_flow_label(&mut self) -> Result<LabelDecl, ParseError> {
        self.consume(TokenKind::At, "Expected '@' for control flow label")?;
        let name = self.consume(TokenKind::Identifier, "Expected label name after '@'")?;

        self.consume(
            TokenKind::LeftBracket,
            "Expected '[' to start control flow label body",
        )?;
        let body = self.block_statements()?;

        Ok(LabelDecl::ControlFlow(ControlFlowLabel { name, body }))
    }

    // =========================================================
    // Enum + Switch (parser-enum.js, parser-switch.js) :contentReference[oaicite:19]{index=19} :contentReference[oaicite:20]{index=20}
    // =========================================================

    fn enum_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(
            TokenKind::Identifier,
            "Expected enum name after 'enum' keyword",
        )?;
        self.consume(TokenKind::LeftBracket, "Expected '[' after enum name")?;

        let mut members: Vec<EnumMember> = vec![];
        let mut auto_value: i64 = 0;

        while !self.check(TokenKind::RightBracket) && !self.is_at_end() {
            let member_name = self.consume(TokenKind::Identifier, "Expected enum member name")?;
            let value: i64;

            if self.match_any(&[TokenKind::Equal]) {
                let value_tok =
                    self.consume(TokenKind::Number, "Expected number value for enum member")?;
                value = extract_i64_number(&value_tok).unwrap_or(auto_value);
                auto_value = value + 1;
            } else {
                value = auto_value;
                auto_value += 1;
            }

            members.push(EnumMember {
                name: member_name,
                value,
            });

            if !self.check(TokenKind::RightBracket) {
                if !self.match_any(&[TokenKind::Comma]) && !self.check(TokenKind::RightBracket) {
                    return Err(self.error(
                        self.peek().clone(),
                        "Expected ',' between enum members or ']' to close enum",
                    ));
                }
            }
        }

        self.consume(TokenKind::RightBracket, "Expected ']' after enum members")?;
        if members.is_empty() {
            return Err(self.error(
                self.previous().clone(),
                "Enum must have at least one member",
            ));
        }

        Ok(Stmt::EnumDecl { name, members })
    }

    fn switch_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'switch'")?;
        let discriminant = self.expression()?;
        self.consume(
            TokenKind::RightParen,
            "Expected ')' after switch expression",
        )?;

        self.consume(TokenKind::LeftBracket, "Expected '[' to start switch body")?;

        let mut cases: Vec<CaseClause> = vec![];
        let mut default_case: Option<DefaultClause> = None;

        while !self.check(TokenKind::RightBracket) && !self.is_at_end() {
            if self.match_any(&[TokenKind::Case]) {
                let case_value = self.expression()?;
                self.consume(
                    TokenKind::Colon,
                    "Expected ':' in switch-case to make a proper case-based switch",
                )?;
                self.consume(TokenKind::LeftBracket, "Expected '[' after case value")?;
                let body = self.block_statements()?;
                cases.push(CaseClause {
                    value: case_value,
                    body,
                });
            } else if self.match_any(&[TokenKind::Default]) {
                if default_case.is_some() {
                    return Err(self.error(
                        self.previous().clone(),
                        "Switch statement can only have one 'default' case",
                    ));
                }
                self.consume(
                    TokenKind::Colon,
                    "Expected ':' in switch-case to make a proper default-based switch",
                )?;
                self.consume(TokenKind::LeftBracket, "Expected '[' after 'default'")?;
                let body = self.block_statements()?;
                default_case = Some(DefaultClause { body });
            } else {
                return Err(self.error(
                    self.peek().clone(),
                    "Expected 'case' or 'default' in switch body",
                ));
            }
        }

        self.consume(TokenKind::RightBracket, "Expected ']' after switch body")?;
        if cases.is_empty() && default_case.is_none() {
            return Err(self.error(
                self.previous().clone(),
                "Switch statement must have at least one case or default clause",
            ));
        }

        Ok(Stmt::SwitchStmt {
            discriminant,
            cases,
            default_case,
        })
    }

    // =========================================================
    // Expressions (parser-expressions.js) :contentReference[oaicite:21]{index=21}
    // =========================================================

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logical_or()?;

        if self.match_any(&[TokenKind::Equal]) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                Expr::MemberAccess { .. } | Expr::IndexAccess { .. } => {
                    // JS created {type:'AssignmentExpr', target, value}
                    // You can make a dedicated Expr::AssignmentTarget if you want.
                    Ok(Expr::Assign {
                        name: self.previous().clone(),
                        value: Box::new(value),
                    })
                }
                _ => Err(self.error(self.previous().clone(), "Invalid assignment target")),
            }
        } else {
            Ok(expr)
        }
    }

    fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_and()?;
        while self.match_any(&[TokenKind::PipePipe]) {
            let op = self.previous().clone();
            let right = self.logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_any(&[TokenKind::AmpersandAmpersand]) {
            let op = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_any(&[TokenKind::EqualEqual, TokenKind::BangEqual]) {
            let op = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.match_any(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let op = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.match_any(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_any(&[TokenKind::Bang, TokenKind::Minus]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                right: Box::new(right),
            });
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, ParseError> {
        let expr = self.call()?;
        if self.match_any(&[TokenKind::PlusPlus, TokenKind::MinusMinus]) {
            let op = self.previous().clone();
            return Ok(Expr::Postfix {
                operator: op,
                operand: Box::new(expr),
            });
        }
        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_any(&[TokenKind::LeftBracket]) {
                let index = self.expression()?;
                self.consume(TokenKind::RightBracket, "Expected ']' after index")?;
                expr = Expr::IndexAccess {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
                continue;
            }

            if self.match_any(&[TokenKind::Arrow]) {
                let member = self.consume(
                    TokenKind::Identifier,
                    "Expected property or method name after '->'",
                )?;
                if self.check(TokenKind::LeftParen) {
                    self.advance(); // '('
                    let args = self.assoc_function_call()?;
                    expr = Expr::ObjectMethodCall {
                        object: Box::new(expr),
                        method: member,
                        args,
                    };
                } else {
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        property: member,
                    };
                }
                continue;
            }

            if self.match_any(&[TokenKind::Dot]) {
                let variant =
                    self.consume(TokenKind::Identifier, "Expected variant name after '.'")?;
                expr = Expr::EnumAccess {
                    enum_name: Box::new(expr),
                    variant,
                };
                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        // bool literal (token literal)
        if self.check(TokenKind::Bool) {
            let t = self.advance();
            return Ok(Expr::Literal {
                value: literal_from_token(&t)?,
            });
        }

        if self.match_any(&[TokenKind::Log]) {
            return Ok(Expr::Log {
                value: Box::new(self.expression()?),
            });
        }

        if self.match_any(&[TokenKind::Nil]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Null,
            });
        }

        if self.check(TokenKind::Number) {
            let t = self.advance();
            return Ok(Expr::Literal {
                value: literal_from_token(&t)?,
            });
        }

        if self.match_any(&[TokenKind::Command]) {
            return self.command_expression();
        }

        if self.check(TokenKind::String) || self.check(TokenKind::Char) {
            let t = self.advance();
            return Ok(Expr::Literal {
                value: literal_from_token(&t)?,
            });
        }

        // self / identifier
        if self.check(TokenKind::SelfKw) || self.check(TokenKind::Identifier) {
            return self.identifier_identification();
        }

        // bit ops: _and_ etc
        if self.match_any(&[TokenKind::BitAnd]) {
            self.consume(TokenKind::LeftBracket, "Expected '[' to open bit and")?;
            let n1 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(TokenKind::Comma, "Expected ',' to separate bit and numbers")?;
            let n2 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(TokenKind::RightBracket, "Expected ']' to close bit and")?;
            return Ok(Expr::BitAnd { n1, n2 });
        }
        if self.match_any(&[TokenKind::BitOr]) {
            self.consume(TokenKind::LeftBracket, "Expected '[' to open bit or")?;
            let n1 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(TokenKind::Comma, "Expected ',' to separate bit or numbers")?;
            let n2 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(TokenKind::RightBracket, "Expected ']' to close bit or")?;
            return Ok(Expr::BitOr { n1, n2 });
        }
        if self.match_any(&[TokenKind::BitXor]) {
            self.consume(TokenKind::LeftBracket, "Expected '[' to open bit xor")?;
            let n1 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(TokenKind::Comma, "Expected ',' to separate bit xor numbers")?;
            let n2 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(TokenKind::RightBracket, "Expected ']' to close bit xor")?;
            return Ok(Expr::BitXor { n1, n2 });
        }
        if self.match_any(&[TokenKind::BitComp]) {
            self.consume(
                TokenKind::LeftBracket,
                "Expected '[' to open bit complement",
            )?;
            let n1 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(
                TokenKind::RightBracket,
                "Expected ']' to close bit complement",
            )?;
            return Ok(Expr::BitComp { n1 });
        }
        if self.match_any(&[TokenKind::BLShift]) {
            self.consume(
                TokenKind::LeftBracket,
                "Expected '[' to open bit left shift",
            )?;
            let n1 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(
                TokenKind::Comma,
                "Expected ',' to separate bit left shift numbers",
            )?;
            let n2 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(
                TokenKind::RightBracket,
                "Expected ']' to close bit left shift",
            )?;
            return Ok(Expr::BitLShift { n1, n2 });
        }
        if self.match_any(&[TokenKind::BRShift]) {
            self.consume(
                TokenKind::LeftBracket,
                "Expected '[' to open bit right shift",
            )?;
            let n1 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(
                TokenKind::Comma,
                "Expected ',' to separate bit right shift numbers",
            )?;
            let n2 = self.consume(TokenKind::Number, "Expected number")?;
            self.consume(
                TokenKind::RightBracket,
                "Expected ']' to close bit right shift",
            )?;
            return Ok(Expr::BitRShift { n1, n2 });
        }

        if self.match_any(&[TokenKind::LeftParen]) {
            let e = self.expression()?;
            self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
            return Ok(Expr::Grouping {
                expression: Box::new(e),
            });
        }

        if self.match_any(&[TokenKind::New]) {
            return self.object_creation();
        }

        if self.match_any(&[TokenKind::Define]) {
            let v = self.consume(
                TokenKind::Identifier,
                "Expected 'Identifier', while declaring a 'define' variable",
            )?;
            return Ok(Expr::Define { variable: v });
        }

        if self.match_any(&[TokenKind::LeftBracket]) {
            return self.list_literal();
        }

        if self.match_any(&[TokenKind::LeftBrace]) {
            return self.dict_literal();
        }

        if self.match_any(&[TokenKind::SHIF]) {
            self.consume(
                TokenKind::LeftParen,
                "Expected '(' to enclose choose condition",
            )?;
            let condition = self.expression()?;
            self.consume(
                TokenKind::RightParen,
                "Expected ')' to enclose choose condition",
            )?;
            self.consume(
                TokenKind::IdentityOperator,
                "Expected '?' as identity operator in choose condition",
            )?;
            let if_branch = self.expression()?;
            self.consume(
                TokenKind::Colon,
                "Expected ':' to separate choose block to else block",
            )?;
            let else_block = self.expression()?;
            return Ok(Expr::ShifChoose {
                condition: Box::new(condition),
                choose_block: Box::new(if_branch),
                else_block: Box::new(else_block),
            });
        }

        Err(self.error(self.peek().clone(), "Expected expression"))
    }

    fn identifier_identification(&mut self) -> Result<Expr, ParseError> {
        let ident = self.advance(); // Identifier or SelfKw

        // macro invocation expands into a Block of injected Var(...) + macro body :contentReference[oaicite:22]{index=22}
        if self.macro_map.contains_key(&ident.lexeme) {
            let block = self.macro_invocation_statement(&ident)?;
            return Ok(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: LiteralValue::Null,
                }),
            }); // you can return Expr::Block-like; JS returned Block stmt as expression
        }

        // label call: identifier(...)
        if self.check(TokenKind::LeftParen) {
            self.advance(); // '('
            return self.label_call(ident);
        }

        // special variable accessibility: identifier * def/gen/lock/unlock/kill/revive/is_alive OR access_ttv(history)
        if self.check(TokenKind::Star) {
            self.advance(); // '*'

            if self.match_any(&[TokenKind::Def]) {
                return Ok(Expr::DefaultVariableDefaultAccess { variable: ident });
            } else if self.match_any(&[TokenKind::Gen]) {
                return Ok(Expr::DefaultVariableGeneralAccess { variable: ident });
            } else if self.match_any(&[TokenKind::Lock]) {
                return Ok(Expr::DeadlockLock { variable: ident });
            } else if self.match_any(&[TokenKind::Unlock]) {
                return Ok(Expr::DeadlockUnlock { variable: ident });
            } else if self.match_any(&[TokenKind::Kill]) {
                return Ok(Expr::DeadlockKill { variable: ident });
            } else if self.match_any(&[TokenKind::Revive]) {
                return Ok(Expr::DeadlockRevive { variable: ident });
            } else if self.match_any(&[TokenKind::IsAlive]) {
                return Ok(Expr::DeadlockIsAlive { variable: ident });
            } else {
                let past = self.expression()?;
                return Ok(Expr::AccessTtv {
                    variable: ident,
                    history: Box::new(past),
                });
            }
        }

        Ok(Expr::Variable { name: ident })
    }

    fn label_call(&mut self, name: Token) -> Result<Expr, ParseError> {
        let mut args: HashMap<String, Expr> = HashMap::new();

        while !self.check(TokenKind::RightParen) && !self.is_at_end() {
            let key = self.consume(TokenKind::Identifier, "Expected 'IDENTIFIER' in label call to map arguments corresponding to its parameter")?;
            self.consume(
                TokenKind::Equal,
                "Expected '=' to map argument key corresponding to its value",
            )?;
            let value = self.expression()?;
            args.insert(key.lexeme.clone(), value);

            if self.match_any(&[TokenKind::Comma]) {
                continue;
            }
            break;
        }

        self.consume(
            TokenKind::RightParen,
            "Expected ')' to enclose label argument environment.",
        )?;
        Ok(Expr::LabelCall { name, args })
    }

    fn assoc_function_call(&mut self) -> Result<HashMap<String, Expr>, ParseError> {
        let mut args: HashMap<String, Expr> = HashMap::new();

        while !self.check(TokenKind::RightParen) && !self.is_at_end() {
            let key = self.consume(TokenKind::Identifier, "Expected 'IDENTIFIER' in function call to map arguments corresponding to its parameter")?;
            self.consume(
                TokenKind::Equal,
                "Expected '=' to map argument key corresponding to its value",
            )?;
            let value = self.expression()?;
            args.insert(key.lexeme.clone(), value);

            if self.match_any(&[TokenKind::Comma]) {
                continue;
            }
            break;
        }

        self.consume(
            TokenKind::RightParen,
            "Expected ')' to enclose function argument environment.",
        )?;
        Ok(args)
    }

    fn macro_invocation_statement(&mut self, identifier: &Token) -> Result<Vec<Stmt>, ParseError> {
        let m = self.macro_map.get(&identifier.lexeme).cloned().unwrap();
        let mut expr_args: Vec<Expr> = vec![];

        if self.match_any(&[TokenKind::LeftParen]) {
            while !self.is_at_end() && !self.check(TokenKind::RightParen) {
                expr_args.push(self.expression()?);
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }
            self.consume(
                TokenKind::RightParen,
                "Expected ')' to enclose macro's calling with arguments",
            )?;
        }

        let mut block: Vec<Stmt> = vec![];
        if !expr_args.is_empty() && !m.params.is_empty() {
            for (i, src) in expr_args.into_iter().enumerate() {
                if let Some(param_tok) = m.params.get(i) {
                    block.push(Stmt::Var {
                        name: param_tok.clone(),
                        initializer: src,
                    });
                }
            }
        }
        block.extend(m.body);
        Ok(block)
    }

    fn object_creation(&mut self) -> Result<Expr, ParseError> {
        self.consume(
            TokenKind::ColonColon,
            "Expected '::' after new keyword, while creating new Instance of Structure",
        )?;
        let ident = self.consume(
            TokenKind::Identifier,
            "Expected Instance name as Identifier",
        )?;
        self.consume(
            TokenKind::LeftParen,
            "Expected '(' to start enclosing constructor arguments",
        )?;

        let mut args: HashMap<String, Expr> = HashMap::new();
        while !self.check(TokenKind::RightParen) && !self.is_at_end() {
            let key = self.consume(TokenKind::Identifier, "Expected 'IDENTIFIER' in constructor call to map arguments corresponding to its parameter")?;
            self.consume(
                TokenKind::Equal,
                "Expected '=' to map argument key corresponding to its value",
            )?;
            let value = self.expression()?;
            args.insert(key.lexeme.clone(), value);

            if self.match_any(&[TokenKind::Comma]) {
                continue;
            }
            break;
        }
        self.consume(
            TokenKind::RightParen,
            "Expected ')' to enclose constructor argument environment.",
        )?;

        Ok(Expr::ObjectCreation { name: ident, args })
    }

    fn list_literal(&mut self) -> Result<Expr, ParseError> {
        let mut elements: Vec<Expr> = vec![];
        if !self.check(TokenKind::RightBracket) {
            loop {
                elements.push(self.expression()?);
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }
        }
        self.consume(TokenKind::RightBracket, "Expected ']' after list elements")?;
        Ok(Expr::ListLiteral { elements })
    }

    fn dict_literal(&mut self) -> Result<Expr, ParseError> {
        let mut entries: Vec<(Expr, Expr)> = vec![];
        if !self.check(TokenKind::RightBrace) {
            loop {
                let key = self.expression()?;
                self.consume(TokenKind::Colon, "Expected ':' after dict key")?;
                let value = self.expression()?;
                entries.push((key, value));
                if self.match_any(&[TokenKind::Comma]) {
                    continue;
                }
                break;
            }
        }
        self.consume(TokenKind::RightBrace, "Expected '}' after dict entries")?;
        Ok(Expr::DictLiteral { entries })
    }

    fn command_expression(&mut self) -> Result<Expr, ParseError> {
        let mut parts: Vec<Expr> = vec![];

        // JS parses until Dot encountered :contentReference[oaicite:23]{index=23}
        while !self.is_at_end() && !self.match_any(&[TokenKind::Dot]) {
            // Parenthesized expression inside command
            if self.match_any(&[TokenKind::LeftParen]) {
                let e = self.expression()?;
                self.consume(
                    TokenKind::RightParen,
                    "Expected ')' after command expression",
                )?;
                parts.push(e);
                continue;
            }

            // Flags: collect - / -- sequences then attach next token text
            if self.check(TokenKind::Minus) || self.check(TokenKind::MinusMinus) {
                let mut flag = String::new();
                while !self.is_at_end() && !self.check(TokenKind::Dot) {
                    if self.match_any(&[TokenKind::MinusMinus]) {
                        flag.push_str("--");
                    } else if self.match_any(&[TokenKind::Minus]) {
                        flag.push('-');
                    } else {
                        break;
                    }
                }

                if !self.is_at_end() && !self.check(TokenKind::Dot) {
                    let nxt = self.peek().clone();
                    if matches!(
                        nxt.kind,
                        TokenKind::Identifier
                            | TokenKind::Number
                            | TokenKind::String
                            | TokenKind::Char
                    ) {
                        flag.push_str(&token_as_string(&nxt));
                        self.advance();
                    }
                }

                parts.push(Expr::Literal {
                    value: LiteralValue::String(flag),
                });
                continue;
            }

            // General token -> string literal piece
            let tok = self.peek().clone();
            self.advance();
            let text = token_as_string(&tok);
            parts.push(Expr::Literal {
                value: LiteralValue::String(text),
            });
        }

        Ok(Expr::Command { parts })
    }
}

// =========================
// Internal enums + helpers
// =========================

#[derive(Debug, Clone)]
enum TopLevel {
    Visible(VisibleBlock),
    Label(LabelDecl),
    Struct(StructDecl),
    Stmt(Stmt),
}

fn literal_from_token(t: &Token) -> Result<LiteralValue, ParseError> {
    if let Some(lit) = &t.literal {
        Ok(LiteralValue::from(lit))
    } else if t.kind == TokenKind::Nil {
        Ok(LiteralValue::Null)
    } else {
        Err(ParseError::new(
            t.clone(),
            "Expected literal token to have a literal value",
        ))
    }
}

fn extract_i64_number(t: &Token) -> Option<i64> {
    match &t.literal {
        Some(crate::lexer::Literal::Number(n)) => match n {
            crate::lexer::tokens::NumberLit::Int(v) => i64::try_from(*v).ok(),
            crate::lexer::tokens::NumberLit::Float(v) => Some(*v as i64),
            crate::lexer::tokens::NumberLit::BigIntString(_) => None,
        },
        _ => None,
    }
}

fn token_as_string(t: &Token) -> String {
    if let Some(lit) = &t.literal {
        match lit {
            crate::lexer::Literal::String(s) => s.clone(),
            crate::lexer::Literal::Char(c) => c.to_string(),
            crate::lexer::Literal::Bool(b) => b.to_string(),
            crate::lexer::Literal::Number(n) => match n {
                crate::lexer::tokens::NumberLit::Int(v) => v.to_string(),
                crate::lexer::tokens::NumberLit::Float(v) => v.to_string(),
                crate::lexer::tokens::NumberLit::BigIntString(s) => s.clone(),
            },
            crate::lexer::Literal::Identifier(s) => s.clone(),
        }
    } else {
        t.lexeme.clone()
    }
}
