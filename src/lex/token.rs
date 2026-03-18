use super::Number;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    LPar,
    RPar,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    ShiftLeft,
    ShiftRight,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    Dec,
    Inc,

    Dot,
    Comma,
    Colon,
    Semicolon,
    QuestionMark,
    At,
    Octothorp,
    Dollar,

    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Equals,
    NotEquals,

    Assignment,

    ModuloEq,
    Percent,
    DivideEq,
    TimesEq,
    MinusEq,
    PlusEq,
    RangeInclusive,
    RangeExclusive,
    SmallRightArrow,
    BigRightArrow,
    OrEq,
    AndEq,
    XorEq,
    ShiftRightEq,
    ShiftLeftEq,

    Fn,
    Static,
    Return,
    If,
    Else,
    While,
    Loop,
    Let,
    For,
    As,
    Const,
    Mut,
    Break,
    Continue,

    Struct,
    Enum,
    Union,

    Label(&'a str),
    Ident(&'a str),

    StringLiteral(StringLiteral<'a>),
    CharLiteral(StringLiteral<'a>),
    NumericLiteral(Number<'a>),

    FalseLiteral,
    TrueLiteral,

    SingleLineComment(&'a str),
    MultiLineComment(&'a str),

    Eof,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(Rust, packed)]
pub struct StringLiteral<'a> {
    pub repr: &'a str,
    pub escaped: bool,
}

impl<'a> StringLiteral<'a> {
    pub fn new(repr: &'a str) -> Self {
        Self {
            repr,
            escaped: false,
        }
    }

    pub fn escaped(repr: &'a str) -> Self {
        Self {
            repr,
            escaped: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub span: Span,
    pub val: T,
}

impl<T> Spanned<T> {
    pub fn new(val: T, span: Span) -> Self {
        Self { val, span }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
}
