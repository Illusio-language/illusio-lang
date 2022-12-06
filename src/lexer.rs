use crate::{error::Error, string_spliter, token::*};
use colored::Colorize;
/// # **Illusio Lexer**

pub struct Lexer {
    input: Vec<u8>,
    pub position: usize,
    pub errors: Vec<Error>,
    ch: u8,
    file_name: String,
    pub read_position: usize,
}
impl Lexer {
    /// Create a new Lexer, takes input of string.
    pub fn new(input: &str, file_name: &str) -> Self {
        let mut lexer = Self {
            input: input.as_bytes().to_vec(),
            position: 0,
            file_name: file_name.to_owned(),
            ch: b'\0',
            errors: Vec::new(),
            read_position: 0,
        };
        lexer.read();
        lexer
    }
    /// *read* function takes skips the current token
    fn read(&mut self) {
        // Checking If out current read position passed the input length.
        if self.read_position >= self.input.len() {
            self.ch = b'\0'
        } else {
            self.ch = self.input[self.read_position]
        }
        self.position = self.read_position;
        // NOTE: Read position can also represent the peek token,
        // since we assign position before we increase read_position
        self.read_position += 1;
    }
    /// Used for skiping whitespace
    fn whitespace(&mut self) {
        loop {
            match self.ch {
                b' ' | b'\t' | b'\r' => self.read(),
                b'\n' => self.read(),
                b'#' => {
                    if self.next_match(b'*') {
                        self.read();
                        loop {
                            if self.ch == b'\0' {
                                break;
                            }
                            if self.ch == b'*' && self.peek_next() == Some(b'#') {
                                self.next();
                                break
                            }
                            self.read();
                        }
                    } else {
                        loop {
                            // Reading until the end of the line
                            self.read();
                            if self.ch == b'\n' || self.ch == b'\0' {
                                break;
                            }
                        }
                    }
                }
                _ => break,
            }
        }
    }
    /// Peeking for the next byte
    pub fn peek_next(&self) -> Option<u8> {
        return self.input.get(self.read_position).copied();
    }
    pub fn next(&mut self) -> Token {
        self.whitespace();
        let mut token = Token::new(
            TokenKind::Illegal,
            String::from_utf8(vec![self.ch]).unwrap_or("".to_owned()),
            Span::from(self.position..self.position + 1),
        );
        match self.ch {
            b'+' => {
                token.kind = TokenKind::Plus;
            }
            b'-' => {
                token.kind = TokenKind::Minus;
            }
            b'*' => {
                token.kind = TokenKind::Asterisk;
            }
            b'/' => {
                token.kind = TokenKind::Slash;
            }
            b'%' => {
                token.kind = TokenKind::Mod;
            }
            b'|' => {
                token.kind = TokenKind::Or;
            }
            b'>' => {
                if self.next_match(b'=') {
                    token.kind = TokenKind::GreaterThanEqual;
                    token.literal = String::from(">=");
                    token.span = Span::from(token.span.start..self.position + 1);
                } else {
                    token.kind = TokenKind::GreaterThan;
                }
            }
            b'<' => {
                if self.next_match(b'=') {
                    token.kind = TokenKind::LessThanEqual;
                    token.literal = String::from("<=");
                    token.span = Span::from(token.span.start..self.position + 1);
                } else {
                    token.kind = TokenKind::LessThan;
                }
            }
            b'(' => {
                token.kind = TokenKind::OpeningParen;
            }
            b')' => {
                token.kind = TokenKind::ClosingParen;
            }
            b'{' => {
                token.kind = TokenKind::OpeningBracket;
            }
            b':' => {
                token.kind = TokenKind::Colon;
            }
            b'}' => {
                token.kind = TokenKind::ClosingBracket;
            }
            b',' => {
                token.kind = TokenKind::Comma;
            }
            b'=' => {
                if self.next_match(b'=') {
                    token.kind = TokenKind::EqualTo;
                    token.literal = String::from("==");
                    token.span = Span::from(token.span.start..self.position + 1);
                } else {
                    token.kind = TokenKind::Equal;
                }
            }
            b'!' => {
                if self.next_match(b'=') {
                    token.kind = TokenKind::NotEqual;
                    token.literal = String::from("!=");
                    token.span = Span::from(token.span.start..self.position + 1);
                } else {
                    token.kind = TokenKind::Not;
                }
            }

            b'\0' => {
                token.kind = TokenKind::Eof;
                token.literal = "end of file".to_owned()
            }
            b';' => {
                token.kind = TokenKind::SemiColon;
            }
            b'&' => {
                token.kind = TokenKind::And;
            }
            b'\'' => {
                let start = self.position;
                self.read();
                let mut buf: Vec<u8> = Vec::new();
                while self.ch != b'\'' {
                    if self.ch == b'\\' {
                        self.read();
                        if self.ch == b'\'' {
                            buf.push(b'\'');
                            self.read();
                            continue;
                        }
                    }
                    if self.ch == b'\0' {
                        self.errors.push(Error {
                            source: String::from_utf8(self.input.clone()).unwrap(),
                            file_name: self.file_name.clone(),
                            message: "Unterminated String".to_string(),
                            span: Span::from(self.position..self.position + 1),
                            help: format!(
                                "Try adding \"'\" here like this: {}",
                                format!(
                                    "'{}'",
                                    String::from_utf8(buf.clone()).unwrap().green().bold()
                                )
                                .green()
                            ),
                        });
                        break;
                    }
                    buf.push(self.ch);
                    self.read();
                }
                token.span = Span::from(start..self.position + 1);
                let str_buf =
                    String::from_utf8(string_spliter::split_string(buf)).unwrap_or("".to_owned());
                token.kind = TokenKind::String;
                token.literal = str_buf;
            }

            b'"' => {
                let start = self.position;
                self.read();
                let mut buf: Vec<u8> = Vec::new();
                while self.ch != b'"' {
                    if self.ch == b'\\' {
                        if self.peek_next() == Some(b'"') {
                            self.read();
                            buf.push(b'"');
                            self.read();
                            continue;
                        }
                    }
                    if self.ch == b'\0' {
                        self.errors.push(Error {
                            source: String::from_utf8(self.input.clone()).unwrap(),
                            file_name: self.file_name.clone(),
                            message: "Unterminated String".to_string(),
                            span: Span::from(self.position..self.position + 1),
                            help: format!(
                                "Try adding '\"' here like this: {}",
                                format!("\"{}\"", String::from_utf8(buf).unwrap().green().bold())
                                    .green()
                            ),
                        });
                        return token;
                    }
                    buf.push(self.ch);
                    self.read();
                }
                token.span = Span::from(start..self.position + 1);
                let str_buf =
                    String::from_utf8(string_spliter::split_string(buf)).unwrap_or("".to_owned());
                token.kind = TokenKind::String;
                token.literal = str_buf;
            }

            _ => {
                if self.ch.is_ascii_digit()
                    || (self.ch == b'.' && self.peek_next().unwrap_or(0).is_ascii_digit())
                {
                    let mut buf: Vec<u8> = Vec::new();
                    let mut token_type = TokenKind::Int;
                    let start = self.position;
                    let mut accept = "0123456789_";
                    if self.ch == b'0' && self.peek_next().unwrap_or(b'\0') == b'x' {
                        accept = "0x123456789ABCDEFabcdef_"
                    }
                    if self.ch == b'0' && self.peek_next().unwrap_or(b'\0') == b'b' {
                        accept = "0b10_"
                    }
                    while accept.contains(std::str::from_utf8(&[self.ch]).unwrap()) {
                        if self.ch != b'_' {
                            buf.push(self.ch);
                            self.read();
                        } else {
                            self.read()
                        }
                    }
                    if self.ch == b'.' && self.peek_next().unwrap_or(0).is_ascii_digit() {
                        // Pushing the dot
                        buf.push(self.ch);
                        self.read();
                        // Pushing the Number
                        while self.ch.is_ascii_digit() {
                            buf.push(self.ch);
                            self.read();
                        }
                        token_type = TokenKind::Float;
                    } else if self.ch == b'.' && !self.peek_next().unwrap_or(b'\0').is_ascii_digit()
                    {
                        buf.push(self.ch);
                        self.read();
                        token_type = TokenKind::Float;
                    }
                    let str_buf = String::from_utf8(buf).unwrap();
                    token.kind = token_type;
                    token.literal = str_buf;
                    token.span = Span::from(start..self.position);
                    return token;
                }
                if self.ch.is_ascii_alphabetic() || self.ch == b'_' {
                    let mut buf: Vec<u8> = Vec::new();
                    let start = self.position;
                    while self.ch.is_ascii_alphanumeric() || self.ch == b'_' {
                        buf.push(self.ch);
                        self.read();
                    }
                    let str_buf = String::from_utf8(buf).unwrap();
                    token.kind = self.keyword(&str_buf);
                    token.literal = str_buf;
                    token.span = Span::from(start..self.position);
                    return token;
                }
            }
        }
        self.read();
        return token;
    }
    fn next_match(&mut self, expected: u8) -> bool {
        if self.peek_next() == Some(expected) {
            self.read();
            return true;
        }
        false
    }
    fn keyword(&self, literal: &str) -> TokenKind {
        match literal {
            "int" => TokenKind::IntTy,
            "float" => TokenKind::FloatTy,
            "str" => TokenKind::StringTy,
            "if" => TokenKind::IfKw,
            "fun" => TokenKind::Fun,
            "bool" => TokenKind::BoolTy,
            "enum" => TokenKind::Enum,
            "end" => TokenKind::End,
            "do" => TokenKind::Do,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Identifier,
        }
    }
}
