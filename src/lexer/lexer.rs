use crate::lexer::errors::LexError;
use crate::lexer::tokens::{Literal, NumberLit, Token, TokenKind};

pub struct Lexer {
    src: Vec<char>,
    source: String,

    start: usize,
    current: usize,

    line: usize,   // 1-based
    column: usize, // 1-based (column of *current* position after advance)
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        let src: Vec<char> = source.chars().collect();
        Self {
            src,
            source,
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, LexError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_code_token()?;
        }
        self.tokens.push(Token::new(TokenKind::Eof, "", self.line));
        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }

    fn advance(&mut self) -> char {
        let c = self.src[self.current];
        self.current += 1;

        // Maintain line/column similar to JS line tracking, but we also track column.
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column = self.column.saturating_add(1);
        }
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.src[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.src.len() {
            '\0'
        } else {
            self.src[self.current + 1]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.src[self.current] != expected {
            return false;
        }
        self.current += 1;
        self.column = self.column.saturating_add(1);
        true
    }

    fn lexeme(&self) -> String {
        self.src[self.start..self.current].iter().collect()
    }

    fn add_token(&mut self, kind: TokenKind) {
        let text = self.lexeme();
        self.tokens.push(Token::new(kind, text, self.line));
    }

    fn add_value_token(&mut self, kind: TokenKind, lit: Literal) {
        let text = self.lexeme();
        self.tokens
            .push(Token::with_literal(kind, text, self.line, lit));
    }

    fn err(&self, msg: impl Into<String>) -> LexError {
        // column in JS error was basically “current” index; here we provide real column.
        LexError::new(self.line, self.column.saturating_sub(1).max(1), msg)
    }

    fn scan_code_token(&mut self) -> Result<(), LexError> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '?' => self.add_token(TokenKind::IdentityOperator),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            '[' => self.add_token(TokenKind::LeftBracket),
            ']' => self.add_token(TokenKind::RightBracket),
            '#' => self.add_token(TokenKind::Hash),
            ',' => self.add_token(TokenKind::Comma),

            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenKind::And);
                } else {
                    self.add_token(TokenKind::Ampersand);
                }
            }

            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenKind::PipePipe);
                } else {
                    self.add_token(TokenKind::Or);
                }
            }

            '+' => {
                if self.match_char('+') {
                    self.add_token(TokenKind::PlusPlus);
                } else {
                    self.add_token(TokenKind::Plus);
                }
            }

            '-' => {
                if self.match_char('-') {
                    self.add_token(TokenKind::MinusMinus);
                } else if self.match_char('>') {
                    self.add_token(TokenKind::Arrow);
                } else if self.peek().is_ascii_digit() {
                    self.negative_number()?;
                } else {
                    self.add_token(TokenKind::Minus);
                }
            }

            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::EqualEqual);
                } else {
                    self.add_token(TokenKind::Equal);
                }
            }

            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BangEqual);
                } else {
                    self.add_token(TokenKind::Bang);
                }
            }

            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::LessEqual);
                } else {
                    self.add_token(TokenKind::Less);
                }
            }

            ':' => {
                if self.match_char(':') {
                    self.add_token(TokenKind::ColonColon);
                } else {
                    self.add_token(TokenKind::Colon);
                }
            }

            '>' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::Command);
                } else if self.match_char('=') {
                    self.add_token(TokenKind::GreaterEqual);
                } else {
                    self.add_token(TokenKind::Greater);
                }
            }

            '*' => self.add_token(TokenKind::Star),
            '%' => self.add_token(TokenKind::Percent),
            '.' => self.add_token(TokenKind::Dot),
            '@' => self.add_token(TokenKind::At),

            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    while !self.is_at_end() {
                        if self.peek() == '*' && self.peek_next() == '/' {
                            self.advance(); // '*'
                            self.advance(); // '/'
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }

            '\'' => self.char_literal()?,

            '"' => self.string_literal()?,

            ' ' | '\r' | '\t' => { /* ignore */ }
            '\n' => { /* line already handled in advance() */ }

            _ => {
                if c.is_ascii_digit() || c == 'O' {
                    self.type_or_number(c)?;
                } else if is_ident_start(c) {
                    self.identifier(c);
                } else {
                    return Err(self.err(format!("Unexpected character: '{}'", c)));
                }
            }
        }

        Ok(())
    }

    fn char_literal(&mut self) -> Result<(), LexError> {
        if self.is_at_end() {
            return Err(self.err("Unterminated character literal"));
        }

        let ch: char;

        if self.peek() == '\\' {
            self.advance(); // '\'
            if self.is_at_end() {
                return Err(self.err("Unterminated escape sequence in character literal"));
            }
            let escaped = self.advance();
            ch = match escaped {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '0' => '\0',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                'x' => {
                    let h1 = self.advance();
                    let h2 = self.advance();
                    if !is_hex(h1) || !is_hex(h2) {
                        return Err(self.err("Incomplete or invalid hex escape sequence"));
                    }
                    let v = u8::from_str_radix(&format!("{h1}{h2}"), 16)
                        .map_err(|_| self.err("Invalid hex escape"))?;
                    v as char
                }
                'u' => {
                    if self.peek() != '{' {
                        return Err(self.err("Expected '{' after \\u"));
                    }
                    self.advance(); // '{'
                    let mut hex = String::new();
                    while !self.is_at_end() && self.peek() != '}' {
                        let nc = self.peek();
                        if !is_hex(nc) {
                            return Err(self.err("Invalid character in unicode escape"));
                        }
                        hex.push(self.advance());
                        if hex.len() > 6 {
                            return Err(self.err("Unicode escape sequence too long"));
                        }
                    }
                    if self.is_at_end() || self.peek() != '}' {
                        return Err(self.err("Unterminated unicode escape sequence"));
                    }
                    self.advance(); // '}'
                    if hex.is_empty() {
                        return Err(self.err("Empty unicode escape sequence"));
                    }
                    let cp = u32::from_str_radix(&hex, 16)
                        .map_err(|_| self.err("Invalid unicode code point"))?;
                    char::from_u32(cp).ok_or_else(|| self.err("Invalid unicode code point"))?
                }
                _ => return Err(self.err(format!("Unknown escape sequence: \\{}", escaped))),
            };
        } else {
            if self.peek() == '\n' || self.peek() == '\r' {
                return Err(self.err("Character literal cannot contain newline"));
            }
            ch = self.advance();
        }

        if self.is_at_end() || self.peek() != '\'' {
            return Err(self.err("Expected closing ' after character literal"));
        }
        self.advance(); // closing '

        // lexeme includes quotes
        self.add_value_token(TokenKind::Char, Literal::Char(ch));
        Ok(())
    }

    fn string_literal(&mut self) -> Result<(), LexError> {
        let mut value = String::new();

        // triple quoted multiline """ ... """
        let is_multiline = self.peek() == '"' && self.peek_next() == '"';
        if is_multiline {
            self.advance(); // second "
            self.advance(); // third "
        }

        loop {
            if self.is_at_end() {
                let term = if is_multiline { "\"\"\"" } else { "\"" };
                return Err(self.err(format!(
                    "Unterminated string literal (expected closing {term})"
                )));
            }

            if is_multiline {
                if self.peek() == '"' && self.peek_next() == '"' {
                    // need third
                    let third = if self.current + 2 < self.src.len() {
                        self.src[self.current + 2]
                    } else {
                        '\0'
                    };
                    if third == '"' {
                        self.advance();
                        self.advance();
                        self.advance();
                        self.add_value_token(TokenKind::String, Literal::String(value));
                        return Ok(());
                    }
                }
            } else if self.peek() == '"' {
                self.advance(); // closing "
                self.add_value_token(TokenKind::String, Literal::String(value));
                return Ok(());
            }

            if self.peek() == '\\' {
                self.advance(); // '\'
                if self.is_at_end() {
                    return Err(self.err("Unterminated escape sequence in string"));
                }
                let escaped = self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '0' => value.push('\0'),
                    '\\' => value.push('\\'),
                    '\'' => value.push('\''),
                    '"' => value.push('"'),
                    '\n' => {
                        // line continuation: already advanced and line count handled
                    }
                    'x' => {
                        let h1 = self.advance();
                        let h2 = self.advance();
                        if !is_hex(h1) || !is_hex(h2) {
                            return Err(self.err("Incomplete or invalid hex escape sequence"));
                        }
                        let v = u8::from_str_radix(&format!("{h1}{h2}"), 16)
                            .map_err(|_| self.err("Invalid hex escape"))?;
                        value.push(v as char);
                    }
                    'u' => {
                        if self.peek() != '{' {
                            return Err(self.err("Expected '{' after \\u"));
                        }
                        self.advance(); // '{'
                        let mut hex = String::new();
                        while !self.is_at_end() && self.peek() != '}' {
                            let nc = self.peek();
                            if !is_hex(nc) {
                                return Err(self.err("Invalid character in unicode escape"));
                            }
                            hex.push(self.advance());
                            if hex.len() > 6 {
                                return Err(self.err("Unicode escape sequence too long"));
                            }
                        }
                        if self.is_at_end() || self.peek() != '}' {
                            return Err(self.err("Unterminated unicode escape sequence"));
                        }
                        self.advance(); // '}'
                        if hex.is_empty() {
                            return Err(self.err("Empty unicode escape sequence"));
                        }
                        let cp = u32::from_str_radix(&hex, 16)
                            .map_err(|_| self.err("Invalid unicode code point"))?;
                        let ch = char::from_u32(cp)
                            .ok_or_else(|| self.err("Invalid unicode code point"))?;
                        value.push(ch);
                    }
                    _ => return Err(self.err(format!("Unknown escape sequence: \\{}", escaped))),
                }
            } else {
                if self.peek() == '\n' && !is_multiline {
                    return Err(
                        self.err("Unterminated string literal (newline in non-multiline string)")
                    );
                }
                value.push(self.advance());
            }
        }
    }

    fn negative_number(&mut self) -> Result<(), LexError> {
        // start is at '-' already included in lexeme; we’ll parse from chars
        let mut text = String::from("-");
        let mut has_dot = false;

        while !self.is_at_end() {
            let c = self.peek();
            if c == '.' {
                if has_dot {
                    return Err(self.err("Multiple '.' characters in number"));
                }
                if !self.peek_next().is_ascii_digit() {
                    return Err(self.err("Dot must be followed by digit in number"));
                }
                has_dot = true;
                text.push(self.advance());
            } else if c.is_ascii_digit() {
                text.push(self.advance());
            } else {
                break;
            }
        }

        if text == "-" || text == "-." {
            // back up to just after '-': mimic JS fallback to Minus token
            self.current = self.start + 1;
            // column best-effort: recompute from source line not worth here
            self.add_token(TokenKind::Minus);
            return Ok(());
        }

        let lit = parse_number_like_js(&text);
        self.tokens.push(Token::with_literal(
            TokenKind::Number,
            text,
            self.line,
            Literal::Number(lit),
        ));
        Ok(())
    }

    fn type_or_number(&mut self, first: char) -> Result<(), LexError> {
        // base literals: Ox / Ob / Oo (note: your JS uses 'O' not '0') :contentReference[oaicite:3]{index=3}
        if first == 'O' {
            let second = self.peek();
            let (radix, has_prefix) = match second {
                'x' | 'X' => (16u32, true),
                'b' | 'B' => (2u32, true),
                'o' | 'O' => (8u32, true),
                _ => (10u32, false),
            };

            if has_prefix {
                self.advance(); // consume x/b/o
                let mut digits = String::new();

                while !self.is_at_end() {
                    let ch = self.peek();
                    let valid = match radix {
                        16 => is_hex(ch),
                        2 => ch == '0' || ch == '1',
                        8 => ch >= '0' && ch <= '7',
                        _ => false,
                    };
                    if valid {
                        digits.push(self.advance());
                    } else {
                        break;
                    }
                }

                if digits.is_empty() {
                    return Err(self.err("Expected digits after base prefix (Ox, Ob, Oo)"));
                }

                let lexeme = self.lexeme();
                let lit = parse_int_radix_best_effort(&digits, radix);
                self.tokens.push(Token::with_literal(
                    TokenKind::Number,
                    lexeme,
                    self.line,
                    Literal::Number(lit),
                ));
                return Ok(());
            }
        }

        // normal decimal or float
        let mut number_string = String::new();
        number_string.push(first);

        let mut is_float = false;
        let mut has_dot = false;

        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_digit() {
                number_string.push(self.advance());
            } else if ch == '.' {
                if has_dot {
                    return Err(self.err("Multiple '.' in number"));
                }
                if !self.peek_next().is_ascii_digit() {
                    break; // dot operator
                }
                has_dot = true;
                is_float = true;
                number_string.push(self.advance());
            } else {
                break;
            }
        }

        if number_string == "." || number_string.is_empty() {
            return Err(self.err("Invalid number format"));
        }

        let lit = if is_float {
            let v: f64 = number_string
                .parse()
                .map_err(|_| self.err("Invalid float"))?;
            NumberLit::Float(v)
        } else {
            parse_int_best_effort(&number_string)
        };

        self.tokens.push(Token::with_literal(
            TokenKind::Number,
            number_string,
            self.line,
            Literal::Number(lit),
        ));
        Ok(())
    }

    fn identifier(&mut self, first: char) {
        let mut text = String::new();
        text.push(first);

        while !self.is_at_end() && is_ident_continue(self.peek()) {
            text.push(self.advance());
        }

        // Keyword mapping from your JS lexer :contentReference[oaicite:4]{index=4}
        let (kind, literal) = match text.as_str() {
            "import" => (TokenKind::Import, None),
            "label" => (TokenKind::Label, None),
            "if" => (TokenKind::If, None),
            "elif" => (TokenKind::Elif, None),
            "else" => (TokenKind::Else, None),
            "jump" => (TokenKind::Jump, None),
            "unlabel" => (TokenKind::Unlabel, None),
            "visible_soft" => (TokenKind::VisibleSoft, None),
            "visible_hard" => (TokenKind::VisibleHard, None),
            "visibility" => (TokenKind::Visibility, None),
            "struct" => (TokenKind::Struct, None),

            "eternal" => (TokenKind::Eternal, None),
            "rooted" => (TokenKind::Rooted, None),
            "define" => (TokenKind::Define, None),
            "new" => (TokenKind::New, None),
            "return" => (TokenKind::Return, None),

            "constructor" => (TokenKind::Constructor, None),
            "self" => (TokenKind::SelfKw, None),
            "public" => (TokenKind::Public, None),
            "private" => (TokenKind::Private, None),

            "true" => (TokenKind::True, Some(Literal::Bool(true))),
            "false" => (TokenKind::False, Some(Literal::Bool(false))),
            "nil" => (TokenKind::Nil, None),

            "_define_" => (TokenKind::DefineMacro, None),
            "ifdef" => (TokenKind::IfDef, None),
            "ifndef" => (TokenKind::IfNDef, None),
            "undef" => (TokenKind::UnDef, None),

            "enum" => (TokenKind::Enum, None),
            "switch" => (TokenKind::Switch, None),
            "case" => (TokenKind::Case, None),
            "default" => (TokenKind::Default, None),
            "choose" => (TokenKind::SHIF, None),

            "_and_" => (TokenKind::BitAnd, None),
            "_xor_" => (TokenKind::BitXor, None),
            "_or_" => (TokenKind::BitOr, None),
            "_com_" => (TokenKind::BitComp, None),
            "_lsh_" => (TokenKind::BLShift, None),
            "_rsh_" => (TokenKind::BRShift, None),

            "_def_" => (TokenKind::_DEF_, None),
            "def" => (TokenKind::Def, None),
            "gen" => (TokenKind::Gen, None),
            "_ttv_" => (TokenKind::_TTV_, None),
            "_delock_" => (TokenKind::_DELOCK_, None),
            "kill" => (TokenKind::Kill, None),
            "revive" => (TokenKind::Revive, None),
            "is_alive" => (TokenKind::IsAlive, None),
            "lock" => (TokenKind::Lock, None),
            "unlock" => (TokenKind::Unlock, None),
            "log" => (TokenKind::Log, None),

            _ => (
                TokenKind::Identifier,
                Some(Literal::Identifier(text.clone())),
            ),
        };

        self.tokens.push(Token {
            kind,
            lexeme: text,
            line: self.line,
            literal,
        });
    }
}

// ---------- helpers ----------

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn parse_number_like_js(text: &str) -> NumberLit {
    if text.contains('.') {
        match text.parse::<f64>() {
            Ok(v) => NumberLit::Float(v),
            Err(_) => NumberLit::BigIntString(text.to_string()),
        }
    } else {
        parse_int_best_effort(text)
    }
}

fn parse_int_best_effort(text: &str) -> NumberLit {
    // try i128, else keep as string (like JS BigInt fallback idea)
    match text.parse::<i128>() {
        Ok(v) => NumberLit::Int(v),
        Err(_) => NumberLit::BigIntString(text.to_string()),
    }
}

fn parse_int_radix_best_effort(digits: &str, radix: u32) -> NumberLit {
    // try i128 from radix; if overflow, store as string "Ox..." style is already in lexeme,
    // but we keep just digits string as BigIntString for now.
    match i128::from_str_radix(digits, radix) {
        Ok(v) => NumberLit::Int(v),
        Err(_) => NumberLit::BigIntString(format!("(base {radix}) {digits}")),
    }
}
