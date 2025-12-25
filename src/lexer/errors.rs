use std::fmt;

/// Equivalent of JS LexError (and “display(source)”).
#[derive(Debug, Clone)]
pub struct LexError {
    pub line: usize,   // 1-based
    pub column: usize, // 1-based
    pub message: String,
}

impl LexError {
    pub fn new(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            column,
            message: message.into(),
        }
    }

    /// Prints a formatted error message including the source line context
    /// (JS: display(source)).
    pub fn display(&self, source: &str) {
        let lines: Vec<&str> = source.split('\n').collect();
        let context_line = lines.get(self.line.saturating_sub(1)).copied().unwrap_or("");

        let pointer_padding = " ".repeat(self.column.saturating_sub(1));

        eprintln!(
            "\n[Lexer Error] line {}:{} → {}\n   {}\n   {}^",
            self.line, self.column, self.message, context_line, pointer_padding
        );
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}, col {}] {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for LexError {}
