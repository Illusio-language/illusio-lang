use std::f32::consts::E;

use crate::{
    ast::{Expr, ExprKind, Param, Type},
    error::*,
    lexer::Lexer,
    token::{Span, Token, TokenKind},
    traits::Item,
};
pub struct Parser {
    source: String,
    has_lexing_errors: bool,
    pub errors: Vec<Error>,
    lexer: Lexer,
    current: Token,
    filename: String,
    position: usize,
}
impl Parser {
    pub fn new(input: &str, file_name: &str) -> Self {
        let mut p = Parser {
            lexer: Lexer::new(input, file_name),
            current: Token::new(TokenKind::Eof, "\0".to_string(), Span::from(0..0)),
            source: input.to_owned(),
            filename: file_name.to_owned(),
            has_lexing_errors: false,
            errors: Vec::new(),
            position: 0,
        };
        p.next();
        p
    }
    fn next(&mut self) {
        self.position = self.lexer.position;
        self.current = self.lexer.next();
        if !self.lexer.errors.is_empty() && !self.has_lexing_errors {
            self.has_lexing_errors = true;
        }
    }
    fn expect(&mut self, kind: TokenKind) {
        if self.current.kind != kind {
            self.errors.push(Error {
                source: self.source.clone(),
                file_name: self.filename.clone(),
                message: format!("Expected \"{}\" found {}", kind, self.current.literal),
                span: self.current.span,
                help: "".to_owned(),
            });
        }
        self.current = self.lexer.next();
    }

    pub fn parse(&mut self) -> Vec<Expr> {
        let mut exprs = Vec::<Expr>::new();
        loop {
            let expr = self.declaration();
            exprs.push(expr.clone());
            if expr.inner == ExprKind::Eof {
                return exprs;
            }
        }
    }
    fn declaration(&mut self) -> Expr {
        match self.current.kind {
            TokenKind::Eof => Expr {
                inner: ExprKind::Eof,
                span: self.current.span,
            },
            TokenKind::Do => {
                let start = self.position;
                self.next();
                let mut exprs: Vec<Expr> = Vec::new();
                loop {
                    if self.current.kind == TokenKind::End {
                        self.next();
                        break;
                    }
                    if self.current.kind == TokenKind::Eof {
                        self.errors.push(Error {
                            source: self.source.clone(),
                            file_name: self.filename.clone(),
                            message: "Expected `end` at end of block.".to_owned(),
                            span: self.current.span,
                            help: "".to_owned(),
                        });
                        self.next();
                        return Expr {
                            inner: ExprKind::Error,
                            span: self.current.span,
                        };
                    }
                    exprs.push(self.declaration());
                }
                return Expr {
                    inner: ExprKind::Block(exprs),
                    span: Span::from(start..self.position),
                };
            }
            TokenKind::Fun => {
                let start = self.position;
                self.next();
                let name = self.current.literal.clone();
                self.expect(TokenKind::Identifier);

                let mut params: Vec<Param> = vec![];
                if self.current.kind == TokenKind::Identifier {
                    loop {
                        let param_name = self.current.literal.clone();
                        self.next();
                        let param_type = self.parse_type();
                        params.push(Param {
                            param_type,
                            name: param_name,
                        });
                        if self.current.kind == TokenKind::Comma {
                            self.next();
                            continue;
                        } else {
                            break;
                        }
                    }
                }
                let mut return_type: Type = Type::None;
                if self.current.kind == TokenKind::Colon {
                    self.next();
                    return_type = self.parse_type();
                }

                let mut exprs: Vec<Expr> = Vec::new();
                loop {
                    if self.current.kind == TokenKind::End {
                        self.next();
                        break;
                    }
                    if self.current.kind == TokenKind::Eof {
                        self.errors.push(Error {
                            source: self.source.clone(),
                            file_name: self.filename.clone(),
                            message: "Expected `end` at end of block.".to_owned(),
                            span: self.current.span,
                            help: "".to_owned(),
                        });
                        self.next();
                        return Expr {
                            inner: ExprKind::Error,
                            span: self.current.span,
                        };
                    }
                    exprs.push(self.declaration());
                }
                return Expr {
                    inner: ExprKind::FunctionDeclaration(name, params, return_type, exprs),
                    span: Span::from(start..self.position),
                };
            }

            TokenKind::Enum => {
                let start = self.position;
                self.next();
                let name = self.current.literal.clone();
                self.next();
                let mut fields = Vec::<String>::new();
                loop {
                    let str_ = self.current.literal.clone();
                    fields.push(str_);
                    self.next();
                    if self.current.kind == TokenKind::End {
                        self.next();
                        break;
                    }

                    if self.current.kind == TokenKind::Eof {
                        self.errors.push(Error {
                            source: self.source.clone(),
                            file_name: self.filename.clone(),
                            message: "Expected `end` at end of block.".to_owned(),
                            span: self.current.span,
                            help: "".to_owned(),
                        });
                        self.next();
                        return Expr {
                            inner: ExprKind::Error,
                            span: self.current.span,
                        };
                    }
                    if self.current.kind == TokenKind::Comma {
                        self.next();
                        continue;
                    } else {
                        break;
                    }
                }
                self.semicolon();
                return Expr {
                    inner: ExprKind::Enum(name, fields),
                    span: Span::from(start..self.position),
                };
            }
            TokenKind::IfKw => return self.if_expr(),
            _ => {
                let expr = self.parse_expr(0);
                self.semicolon();
                return expr;
            }
        }
    }
    fn if_expr(&mut self) -> Expr {
        let start = self.position;
        self.next();
        let condition = self.parse_expr(0);
        let mut exprs = Vec::<Expr>::new();
        loop {
            if self.current.kind == TokenKind::End {
                self.next();
                break;
            }
            if self.current.kind == TokenKind::Eof {
                self.errors.push(Error {
                    source: self.source.clone(),
                    file_name: self.filename.clone(),
                    message: "Expected `end` at end of block.".to_owned(),
                    span: self.current.span,
                    help: "".to_owned(),
                });
                self.next();
                return Expr {
                    inner: ExprKind::Error,
                    span: self.current.span,
                };
            }
            exprs.push(self.declaration());
        }
        return Expr {
            inner: ExprKind::If(condition.boxed(), exprs),
            span: Span::from(start..self.position),
        };
    }
    fn semicolon(&mut self) {
        if self.current.kind == TokenKind::SemiColon {
            self.next();
        }
    }
    fn parse_expr(&mut self, rbp: i32) -> Expr {
        let start = self.position;
        let tok = self.current.clone();
        let mut left = self.nud(&tok);
        while rbp < self.lbp(&self.current.kind) {
            let tok = self.current.clone();
            self.next();
            let right = self.parse_expr(self.lbp(&tok.kind)).boxed();
            left = self.led(
                left.boxed(),
                tok.kind,
                right,
                Span::from(start..self.position),
            )
        }
        left
    }
    fn lbp(&self, op: &TokenKind) -> i32 {
        match op {
            TokenKind::Mod => 25,
            TokenKind::Plus | TokenKind::Minus => 10,
            TokenKind::Asterisk | TokenKind::Slash => 15,

            TokenKind::GreaterThan
            | TokenKind::LessThan
            | TokenKind::GreaterThanEqual
            | TokenKind::LessThanEqual => 5,
            TokenKind::NotEqual | TokenKind::EqualTo => 3,
            TokenKind::And => 2,
            TokenKind::Or | TokenKind::Colon => 1,
            _ => -1, // In another words stop the expr parsing
        }
    }
    fn led(&self, left: Box<Expr>, op: TokenKind, right: Box<Expr>, span: Span) -> Expr {
        let kind = ExprKind::Binary(left, op, right);
        Expr { inner: kind, span }
    }
    pub fn finish(&self) -> bool {
        if self.has_lexing_errors {
            for err in self.lexer.errors.clone() {
                err.show();
            }
            return true;
        } else if !self.errors.is_empty() {
            for err in self.errors.clone() {
                err.show()
            }
            return true;
        }
        false
    }
    fn parse_type(&mut self) -> Type {
        match self.current.kind {
            TokenKind::IntTy
            | TokenKind::FloatTy
            | TokenKind::StringTy
            | TokenKind::BoolTy
            | TokenKind::Asterisk => {
                let mut ty = match self.current.kind {
                    TokenKind::IntTy => Type::Int,
                    TokenKind::FloatTy => Type::Float,
                    TokenKind::StringTy => Type::String,
                    TokenKind::BoolTy => Type::Bool,

                    _ => unreachable!(),
                };
                self.next();
                if self.current.kind == TokenKind::Asterisk {
                    self.next();
                    ty = Type::Ptr(ty.boxed())
                }

                return ty;
            }
            _ => {
                self.errors.push(Error {
                    source: self.source.clone(),
                    file_name: self.filename.clone(),
                    message: "Expected type".to_string(),
                    span: self.current.span,
                    help: "like `int`, `float` `str` `bool`".to_owned(),
                });
                self.next();
                Type::None
            }
        }
    }

    fn nud(&mut self, tok: &Token) -> Expr {
        match tok.kind {
            TokenKind::String => {
                let _string = self.current.literal.clone();
                let span = self.current.span;
                self.next();
                return Expr {
                    inner: ExprKind::Str(_string),
                    span,
                };
            }
            TokenKind::Int => {
                let span = self.current.span;
                let num = if self.current.literal.contains("x") {
                    let mut chars = self.current.literal.chars();
                    // Skip "0x"
                    chars.next();
                    chars.next();
                    match i64::from_str_radix(chars.as_str(), 16) {
                        Ok(i) => i,
                        Err(err) => match err.kind() {
                            std::num::IntErrorKind::Empty => {
                                self.errors.push(Error {
                                    source: self.source.clone(),
                                    file_name: self.filename.clone(),
                                    message: "Int parsing error: Empty.".to_owned(),
                                    span,
                                    help: "".to_owned(),
                                });
                                self.next();
                                return Expr {
                                    inner: ExprKind::Error,
                                    span,
                                };
                            }
                            std::num::IntErrorKind::InvalidDigit => {
                                self.errors.push(Error {
                                    source: self.source.clone(),
                                    file_name: self.filename.clone(),
                                    message: "Int parsing error: Invalid digit".to_owned(),
                                    span,
                                    help: "".to_owned(),
                                });
                                self.next();
                                return Expr {
                                    inner: ExprKind::Error,
                                    span,
                                };
                            }
                            std::num::IntErrorKind::PosOverflow => {
                                self.errors.push(Error {
                                    source: self.source.clone(),
                                    file_name: self.filename.clone(),
                                    message: "Int parsing error: Positive Overflow".to_owned(),
                                    span,
                                    help: "".to_owned(),
                                });
                                self.next();
                                return Expr {
                                    inner: ExprKind::Error,
                                    span,
                                };
                            }
                            std::num::IntErrorKind::NegOverflow => {
                                self.errors.push(Error {
                                    source: self.source.clone(),
                                    file_name: self.filename.clone(),
                                    message: "Int parsing error: Negative Overflow".to_owned(),
                                    span,
                                    help: "".to_owned(),
                                });
                                self.next();
                                return Expr {
                                    inner: ExprKind::Error,
                                    span,
                                };
                            }
                            std::num::IntErrorKind::Zero => {
                                self.errors.push(Error {
                                    source: self.source.clone(),
                                    file_name: self.filename.clone(),
                                    message: "Int parsing error: Zero".to_owned(),
                                    span,
                                    help: "".to_owned(),
                                });
                                self.next();
                                return Expr {
                                    inner: ExprKind::Error,
                                    span,
                                };
                            }
                            _ => unreachable!(),
                        },
                    }
                } else {
                    self.current.literal.parse().unwrap()
                };

                self.next();
                return Expr {
                    inner: ExprKind::Int(num),
                    span,
                };
            }

            TokenKind::OpeningParen => {
                self.next();
                let expr = self.parse_expr(-1);
                self.expect(TokenKind::ClosingParen);
                return expr;
            }
            TokenKind::Float => {
                let span = self.current.span;

                let float: f64 = match self.current.literal.parse::<f64>() {
                    Err(_) => {
                        self.errors.push(Error {
                            source: self.source.clone(),
                            file_name: self.filename.clone(),
                            message: "Error while parsing float.".to_owned(),
                            span,
                            help: "".to_owned(),
                        });
                        self.next();
                        return Expr {
                            inner: ExprKind::Error,
                            span,
                        };
                    }
                    Ok(val) => val,
                };
                self.next();
                Expr {
                    inner: ExprKind::Float(float),
                    span,
                }
            }
            TokenKind::Plus | TokenKind::Minus | TokenKind::Not => {
                let start = self.position;
                let op = self.current.kind;
                self.next();
                let expr = self.parse_expr(40);
                return Expr {
                    inner: ExprKind::Unary(op, expr.boxed()),
                    span: Span::from(start..self.position),
                };
            }
            TokenKind::And => {
                let start = self.position;
                self.next();
                let expr = self.parse_expr(40);
                return Expr {
                    inner: ExprKind::Ref(expr.boxed()),
                    span: Span::from(start..self.position),
                };
            }
            TokenKind::Bool(b) => {
                let span = self.current.span;
                self.next();
                return Expr {
                    inner: ExprKind::Bool(b),
                    span,
                };
            }
            TokenKind::Identifier => {
                let start = self.position;
                let ident = self.current.literal.clone();
                let span = self.current.span;
                self.next();
                match self.current.kind {
                    TokenKind::IntTy
                    | TokenKind::FloatTy
                    | TokenKind::StringTy
                    | TokenKind::BoolTy => {
                        let ty = self.parse_type();
                        self.expect(TokenKind::Equal);
                        let expr = self.parse_expr(0);
                        return Expr {
                            inner: ExprKind::Var(ident, expr.boxed(), ty),
                            span: Span::from(start + 1..self.lexer.position - 1),
                        };
                    }
                    TokenKind::OpeningParen => {
                        self.next();
                        let mut args = vec![];
                        while self.current.kind != TokenKind::ClosingParen {
                            let arg = self.parse_expr(0);
                            args.push(arg);
                            if self.current.kind == TokenKind::Comma {
                                self.next();
                                continue;
                            } else {
                                break;
                            }
                        }
                        self.expect(TokenKind::ClosingParen);
                        return Expr {
                            inner: ExprKind::FunctionCall(ident, args),
                            span: Span::from(start..self.position),
                        };
                    }
                    _ => Expr {
                        inner: ExprKind::Ident(ident),
                        span,
                    },
                }
            }
            _ => {
                self.errors.push(Error {
                    source: self.source.clone(),
                    file_name: self.filename.clone(),
                    message: "Expected expression".to_owned(),
                    span: self.current.span,
                    help: "".to_owned(),
                });
                self.next();
                return Expr {
                    inner: ExprKind::Error,
                    span: self.current.span,
                };
            }
        }
    }
}
