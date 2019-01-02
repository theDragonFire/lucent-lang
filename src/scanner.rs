pub fn build_scanner(code: &str) -> Scanner {
    let scanner = Scanner {
        current: Token::error("No available token".to_string(), 1, 1), // The tokens have not yet been scanned
        next: Token::error("No available token".to_string(), 1, 1),
        code: code.to_string(),
        index: 0,
        line: 1,
        column: 1,
    };

    scanner.scan_token().scan_token()
}

pub struct Scanner {
    current: Token,
    next: Token,
    code: String,
    index: usize,
    line: u32,
    column: u32,
}

impl Scanner {
    pub fn current_token(&self) -> Token {
        self.current.clone()
    }

    pub fn scan_token(self) -> Self {
        let scanner = Scanner::skip_whitespace(self);
        let index = scanner.index;
        Scanner::scan_token_util(scanner, index, index)
    }

    fn scan_token_util(scanner: Scanner, start: usize, current: usize) -> Self {
        if scanner.is_at_end(current) {
            return Scanner {
                current: scanner.next.clone(),
                next: Token::new(TokenType::EOF, "", scanner.line, scanner.column),
                index: current,
                ..scanner
            };
        }

        match scanner.get_char(current) {
            '(' => scanner.add_token(TokenType::LeftParen, start, current),
            ')' => scanner.add_token(TokenType::RightParen, start, current),
            '{' => scanner.add_token(TokenType::LeftBrace, start, current),
            '}' => scanner.add_token(TokenType::RightBrace, start, current),
            ';' => scanner.add_token(TokenType::Semicolon, start, current),
            ',' => scanner.add_token(TokenType::Comma, start, current),
            '.' => scanner.add_token(TokenType::Dot, start, current),
            '-' => scanner.add_token(TokenType::Minus, start, current),
            '+' => scanner.add_token(TokenType::Plus, start, current),
            '/' => scanner.add_token(TokenType::Slash, start, current),
            '*' => scanner.add_token(TokenType::Star, start, current),
            '!' if scanner.match_char('=', current + 1) => {
                scanner.add_token(TokenType::BangEqual, start, current + 1)
            }
            '!' => scanner.add_token(TokenType::Bang, start, current),
            '=' if scanner.match_char('=', current + 1) => {
                scanner.add_token(TokenType::EqualEqual, start, current + 1)
            }
            '=' => scanner.add_token(TokenType::Equal, start, current),
            '<' if scanner.match_char('=', current + 1) => {
                scanner.add_token(TokenType::LessEqual, start, current + 1)
            }
            '<' => scanner.add_token(TokenType::Less, start, current),
            '>' if scanner.match_char('=', current + 1) => {
                scanner.add_token(TokenType::GreaterEqual, start, current + 1)
            }
            '>' => scanner.add_token(TokenType::Greater, start, current),
            '"' => scanner.string(start, current),
            '1'...'9' => scanner.number(start, current, false),
            _ => {
                let lexeme = scanner.get_lexeme(start, current).to_string();
                scanner.error(
                    start,
                    current,
                    "Invalid character: ".to_string() + lexeme.as_str(),
                )
            }
        }
    }

    fn skip_whitespace(scanner: Scanner) -> Self {
        let (index, line, column) = (scanner.index, scanner.line, scanner.column);
        Scanner::skip_ws_util(scanner, index, line, column)
    }

    fn skip_ws_util(scanner: Scanner, index: usize, line: u32, column: u32) -> Self {
        if scanner.is_at_end(index) {
            scanner.advance_to(index, line, column)
        } else {
            match scanner.get_char(index) {
                ' ' => Scanner::skip_ws_util(scanner, index + 1, line, column + 1),
                '\t' => Scanner::skip_ws_util(scanner, index + 1, line, column + 4),
                '\r' => Scanner::skip_ws_util(scanner, index + 1, line, column + 1),
                '\n' => Scanner::skip_ws_util(scanner, index + 1, line + 1, 1),
                '/' => match scanner.get_char(index + 1) {
                    '/' => Scanner::single_line_comment(scanner, index + 2, line, column + 2),
                    '*' => Scanner::multi_line_comment(scanner, index + 2, line, column + 2, 1),
                    _ => scanner.advance_to(index, line, column),
                },
                _ => scanner.advance_to(index, line, column),
            }
        }
    }

    fn single_line_comment(scanner: Scanner, index: usize, line: u32, column: u32) -> Self {
        if scanner.is_at_end(index) {
            scanner.advance_to(index, line, column)
        } else {
            match scanner.get_char(index) {
                '\n' => Scanner::skip_ws_util(scanner, index + 1, line + 1, 1),
                '\t' => Scanner::single_line_comment(scanner, index + 1, line, column + 4),
                _ => Scanner::single_line_comment(scanner, index + 1, line, column + 1),
            }
        }
    }

    fn multi_line_comment(
        scanner: Scanner,
        index: usize,
        line: u32,
        column: u32,
        nested: u32,
    ) -> Self {
        if scanner.is_at_end(index) {
            scanner.advance_to(index, line, column)
        } else {
            match scanner.get_char(index) {
                '/' => {
                    if scanner.match_char('*', index + 1) {
                        Scanner::multi_line_comment(
                            scanner,
                            index + 2,
                            line,
                            column + 2,
                            nested + 1,
                        )
                    } else {
                        Scanner::multi_line_comment(scanner, index + 1, line, column + 1, nested)
                    }
                }
                '*' => {
                    if scanner.match_char('/', index + 1) && nested > 1 {
                        Scanner::multi_line_comment(
                            scanner,
                            index + 2,
                            line,
                            column + 2,
                            nested - 1,
                        )
                    } else if scanner.match_char('/', index + 1) {
                        Scanner::skip_ws_util(scanner, index + 2, line, column + 2)
                    } else {
                        Scanner::multi_line_comment(scanner, index + 1, line, column + 1, nested)
                    }
                }
                '\n' => Scanner::multi_line_comment(scanner, index + 1, line + 1, 1, nested),
                '\t' => Scanner::multi_line_comment(scanner, index + 1, line, column + 4, nested),
                _ => Scanner::multi_line_comment(scanner, index + 1, line, column + 1, nested),
            }
        }
    }

    fn string(self, start: usize, current: usize) -> Self {
        if self.match_char('"', current + 1) {
            let string = self.get_lexeme(start + 1, current).to_string();
            self.add_token(TokenType::String(string), start, current + 1)
        } else {
            self.string(start, current + 1)
        }
    }

    fn number(self, start: usize, current: usize, decimal_seen: bool) -> Self {
        println!("Shouldn't enter number");
        match self.get_char(current) {
            '0'...'9' => self.number(start, current + 1, decimal_seen),
            '.' if !decimal_seen && !self.is_at_end(current + 1) && self.is_digit(current + 1) => {
                self.number(start, current + 1, true)
            }
            _ => match self.get_lexeme(start, current - 1).parse() {
                Ok(value) => {
                    let line = self.line;
                    let column = self.column + (current - start) as u32;
                    self.add_token(TokenType::Number(value), start, current - 1)
                        .advance_to(current, line, column)
                }
                Err(_) => {
                    let lexeme = self.get_lexeme(start, current - 1).to_string();
                    self.error(
                        start,
                        current - 1,
                        "Unable to parse number: ".to_string() + &lexeme,
                    )
                }
            },
        }
    }

    fn get_char(&self, index: usize) -> char {
        if self.is_at_end(index) {
            '\u{0}'
        } else {
            self.code[index..=index].chars().next().unwrap()
        }
    }

    fn match_char(&self, character: char, index: usize) -> bool {
        !self.is_at_end(index) && self.get_char(index) == character
    }

    fn get_lexeme(&self, start: usize, end: usize) -> &str {
        if !self.is_at_end(end) {
            &self.code[start..=end]
        } else {
            self.get_lexeme(start, end - 1)
        }
    }

    fn advance_to(self, index: usize, line: u32, column: u32) -> Self {
        Scanner {
            index,
            line,
            column,
            ..self
        }
    }

    fn add_token(self, t_type: TokenType, start: usize, current: usize) -> Self {
        let lexeme = self.get_lexeme(start, current);
        Scanner {
            current: self.next.clone(),
            next: Token::new(t_type, lexeme, self.line, self.column),
            index: current + 1,
            column: self.column + (current - start) as u32 + 1,
            ..self
        }
    }

    fn error(self, start: usize, end: usize, message: String) -> Self {
        Scanner {
            current: self.next.clone(),
            next: Token::error(message, self.line, self.column),
            index: end + 1,
            column: self.column + (end - start) as u32,
            ..self
        }
    }

    fn is_digit(&self, index: usize) -> bool {
        self.get_char(index).is_ascii_digit()
    }

    fn is_at_end(&self, index: usize) -> bool {
        index >= self.code.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    t_type: TokenType,
    lexeme: String,
    line: u32,
    column: u32,
}

impl Token {
    fn new(t_type: TokenType, lexeme: &str, line: u32, column: u32) -> Self {
        Token {
            t_type,
            lexeme: lexeme.to_string(),
            line,
            column,
        }
    }

    fn error(message: String, line: u32, column: u32) -> Self {
        Token {
            t_type: TokenType::Error,
            lexeme: message,
            line,
            column,
        }
    }

    pub fn token_type(&self) -> TokenType {
        self.t_type.clone()
    }

    pub fn lexeme(&self) -> String {
        self.lexeme.clone()
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn column(&self) -> u32 {
        self.column
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Question,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    EOF,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_skipping() {
        let scanner = build_scanner(
            " ;\t\r\
// This = should be ignored
/* This /* is /* a */ nested */ comment */;",
        );

        assert_eq!(
            Token::new(TokenType::Semicolon, ";", 1, 2),
            scanner.current_token()
        );
        let scanner = scanner.scan_token();
        assert_eq!(
            Token::new(TokenType::Semicolon, ";", 2, 43),
            scanner.current_token()
        );
        let scanner = scanner.scan_token();
        assert_eq!(
            Token::new(TokenType::EOF, "", 2, 44),
            scanner.current_token()
        );
    }

    #[test]
    fn literal_scanning() {
        // String
        let scanner = build_scanner(" \"Hello\" \"\" ");
        assert_eq!(
            Token::new(TokenType::String("Hello".to_string()), "\"Hello\"", 1, 2),
            scanner.current_token()
        );

        let scanner = scanner.scan_token();
        assert_eq!(
            Token::new(TokenType::String("".to_string()), "\"\"", 1, 10),
            scanner.current_token()
        );

        // Int
        let scanner = build_scanner("123.");
        assert_eq!(
            Token::new(TokenType::Number(123.0), "123", 1, 1),
            scanner.current_token()
        );

        let scanner = scanner.scan_token();
        assert_eq!(
            Token::new(TokenType::Dot, ".", 1, 4),
            scanner.current_token()
        );

        let scanner = build_scanner("123.0");
        assert_eq!(
            Token::new(TokenType::Number(123.0), "123.0", 1, 1),
            scanner.current_token()
        );

        let scanner = build_scanner("123.0.1");
        assert_eq!(
            Token::new(TokenType::Number(123.0), "123.0", 1, 1),
            scanner.current_token()
        );

        let scanner = scanner.scan_token();
        assert_eq!(
            Token::new(TokenType::Dot, ".", 1, 6),
            scanner.current_token()
        );

        let scanner = scanner.scan_token();
        assert_eq!(
            Token::new(TokenType::Number(1.0), "1", 1, 7),
            scanner.current_token()
        );
    }

    #[test]
    fn identifier_recognition() {
        let scanner = build_scanner("");
    }

    #[test]
    #[ignore]
    fn test_string_interpolation() {
        unimplemented!(); // TODO add support for string interpolation
    }
}
