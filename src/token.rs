use std::fmt::Display;

/// Main tokens type enum.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TokenKind {
    // One char tokens
    Plus,
    Minus,
    Not,
    Comma,
    And,
    Colon,
    Asterisk,
    Slash,
    Equal,
    Or,
    OpeningParen,
    ClosingParen,
    OpeningBracket,
    ClosingBracket,
    Mod,
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    SemiColon,
    Illegal,
    // Multiple chars tokens
    String,
    Arrow,
    EqualTo,
    Identifier,
    Bool(bool),
    Fun,
    Do,
    NotEqual,
    Enum,
    Int,
    Float,
    // Keywords
    IntTy,
    FloatTy,
    BoolTy,
    End,
    StringTy,
    IfKw,
    // Eof
    Eof,
}
impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        match self {
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Not => write!(f, "!"),
            NotEqual => write!(f, "!="),
            Colon => write!(f, ":"),
            Asterisk => write!(f, "*"),
            Comma => write!(f, ","),
            Slash => write!(f, "/"),
            EqualTo => write!(f, "=="),
            IntTy => write!(f, "int type"),
            FloatTy => write!(f, "float type"),
            Arrow => write!(f, "->"),
            StringTy => write!(f, "string type"),
            IfKw => write!(f, "if"),
            Enum => write!(f, "enum"),
            Fun => write!(f, "fun"),
            End => write!(f, "end"),
            Do => write!(f, "do"),
            Equal => write!(f, "="),
            And => write!(f, "&"),
            Or => write!(f, "|"),
            SemiColon => write!(f, ";"),
            Mod => write!(f, "%"),
            Bool(b) => write!(f, "{}", b),
            BoolTy => write!(f, "bool type"),
            Identifier => write!(f, "identifier"),
            OpeningParen => write!(f, "("),
            ClosingParen => write!(f, ")"),
            GreaterThan => write!(f, ">"),
            LessThan => write!(f, "<"),
            GreaterThanEqual => write!(f, ">="),
            LessThanEqual => write!(f, "<="),
            OpeningBracket => write!(f, "{{"),
            ClosingBracket => write!(f, "}}"),

            String => write!(f, "string"),
            Int => write!(f, "int"),
            Float => write!(f, "float"),

            Illegal => write!(f, "<illegal>"),
            Eof => write!(f, "<eof>"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
/// Our own Implementation of a span, So we can use Copy derive
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}
impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}
impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
#[derive(Debug, Clone)]
pub struct Token {
    pub literal: String,
    pub kind: TokenKind,
    pub span: Span,
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}@{} {:?}", self.kind, self.span, self.literal)
    }
}
impl Token {
    pub fn new(kind: TokenKind, literal: String, span: Span) -> Self {
        Self {
            kind,
            literal,
            span,
        }
    }
    #[allow(dead_code)]
    pub fn show(&self) {
        println!("{}", self)
    }
}
