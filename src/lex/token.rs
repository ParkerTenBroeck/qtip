use super::Number;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum Token<'a> {
    LPar,
    RPar,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LAngle,
    RAngle,

    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
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

    Assign,
    ModuloAssign,
    DivideAssign,
    TimesAssign,
    MinusAssign,
    PlusAssign,
    OrAssign,
    AndAssign,
    XorAssign,

    Percent,
    RangeInclusive,
    RangeExclusive,
    SmallRightArrow,
    BigRightArrow,

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

    #[default]
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
