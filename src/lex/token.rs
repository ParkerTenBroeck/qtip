use super::Number;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
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
    Or,
    Carrot,
    Tilde,
    AndAnd,
    OrOr,
    Bang,
    Dec,
    Inc,
    ShiftLeft,
    ShiftRight,

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
    Let,
    For,

    If,
    Else,
    While,
    Loop,

    Break,
    Continue,
    Return,

    As,
    Const,
    Mut,

    Mod,
    Use,

    Pub,
    Priv,

    Struct,
    Enum,
    Union,

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

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            match self {
                Self::Ident(_) => return f.write_str("ident"),
                Self::StringLiteral(_)
                | Self::CharLiteral(_)
                | Self::NumericLiteral(_)
                | Self::TrueLiteral
                | Self::FalseLiteral => return f.write_str("literal"),
                Self::SingleLineComment(_) | Self::MultiLineComment(_) => {
                    return f.write_str("comment");
                }
                Self::Eof => return f.write_str("end of file"),
                _ => {}
            }
        }

        match self {
            Self::LPar => display_token_text(f, "("),
            Self::RPar => display_token_text(f, ")"),
            Self::LBrace => display_token_text(f, "{"),
            Self::RBrace => display_token_text(f, "}"),
            Self::LBracket => display_token_text(f, "["),
            Self::RBracket => display_token_text(f, "]"),
            Self::Plus => display_token_text(f, "+"),
            Self::Minus => display_token_text(f, "-"),
            Self::Star => display_token_text(f, "*"),
            Self::Slash => display_token_text(f, "/"),
            Self::Ampersand => display_token_text(f, "&"),
            Self::Or => display_token_text(f, "|"),
            Self::Carrot => display_token_text(f, "^"),
            Self::Tilde => display_token_text(f, "~"),
            Self::AndAnd => display_token_text(f, "&&"),
            Self::OrOr => display_token_text(f, "||"),
            Self::Bang => display_token_text(f, "!"),
            Self::Dec => display_token_text(f, "--"),
            Self::Inc => display_token_text(f, "++"),
            Self::ShiftLeft => display_token_text(f, "<<"),
            Self::ShiftRight => display_token_text(f, ">>"),
            Self::Dot => display_token_text(f, "."),
            Self::Comma => display_token_text(f, ","),
            Self::Colon => display_token_text(f, ":"),
            Self::Semicolon => display_token_text(f, ";"),
            Self::QuestionMark => display_token_text(f, "?"),
            Self::At => display_token_text(f, "@"),
            Self::Octothorp => display_token_text(f, "#"),
            Self::Dollar => display_token_text(f, "$"),
            Self::LessThan => display_token_text(f, "<"),
            Self::LessThanEq => display_token_text(f, "<="),
            Self::GreaterThan => display_token_text(f, ">"),
            Self::GreaterThanEq => display_token_text(f, ">="),
            Self::Equals => display_token_text(f, "=="),
            Self::NotEquals => display_token_text(f, "!="),
            Self::Assign => display_token_text(f, "="),
            Self::ModuloAssign => display_token_text(f, "%="),
            Self::DivideAssign => display_token_text(f, "/="),
            Self::TimesAssign => display_token_text(f, "*="),
            Self::MinusAssign => display_token_text(f, "-="),
            Self::PlusAssign => display_token_text(f, "+="),
            Self::OrAssign => display_token_text(f, "|="),
            Self::AndAssign => display_token_text(f, "&="),
            Self::XorAssign => display_token_text(f, "^="),
            Self::Percent => display_token_text(f, "%"),
            Self::RangeInclusive => display_token_text(f, "..="),
            Self::RangeExclusive => display_token_text(f, ".."),
            Self::SmallRightArrow => display_token_text(f, "->"),
            Self::BigRightArrow => display_token_text(f, "=>"),
            Self::Fn => display_token_text(f, "fn"),
            Self::Static => display_token_text(f, "static"),
            Self::Return => display_token_text(f, "return"),
            Self::If => display_token_text(f, "if"),
            Self::Else => display_token_text(f, "else"),
            Self::While => display_token_text(f, "while"),
            Self::Loop => display_token_text(f, "loop"),
            Self::Let => display_token_text(f, "let"),
            Self::For => display_token_text(f, "for"),
            Self::As => display_token_text(f, "as"),
            Self::Const => display_token_text(f, "const"),
            Self::Mut => display_token_text(f, "mut"),
            Self::Break => display_token_text(f, "break"),
            Self::Continue => display_token_text(f, "continue"),
            Self::Mod => display_token_text(f, "mod"),
            Self::Use => display_token_text(f, "use"),
            Self::Pub => display_token_text(f, "pub"),
            Self::Priv => display_token_text(f, "priv"),
            Self::Struct => display_token_text(f, "struct"),
            Self::Enum => display_token_text(f, "enum"),
            Self::Union => display_token_text(f, "union"),
            Self::Ident(ident) => display_token_text(f, ident),
            Self::StringLiteral(lit) => {
                display_token_text(f, &format!("\"{}\"", lit.repr.escape_debug()))
            }
            Self::CharLiteral(lit) => {
                display_token_text(f, &format!("'{}'", lit.repr.escape_debug()))
            }
            Self::NumericLiteral(number) => display_token_text(f, number.full()),
            Self::FalseLiteral => display_token_text(f, "false"),
            Self::TrueLiteral => display_token_text(f, "true"),
            Self::SingleLineComment(comment) => {
                display_token_text(f, &format!("//{}", comment.escape_debug()))
            }
            Self::MultiLineComment(comment) => {
                display_token_text(f, &format!("/*{}*/", comment.escape_debug()))
            }
            Self::Eof => f.write_str("end of file"),
        }
    }
}

impl<'a> Token<'a> {
    pub fn starts_item(&self) -> bool {
        matches!(
            self,
            Token::Union
                | Token::Struct
                | Token::Enum
                | Token::Static
                | Token::Const
                | Token::Mod
                | Token::Use
                | Token::Fn
                | Token::Pub
                | Token::Priv
                | Token::Ident("extern")
        )
    }

    pub fn starts_stmt(&self) -> bool {
        matches!(
            self,
            Token::Union
                | Token::Struct
                | Token::Enum
                | Token::Static
                | Token::Const
                | Token::Mod
                | Token::Use
                | Token::Fn
                | Token::Pub
                | Token::Priv
                | Token::Ident("extern")
                | Token::Let
                | Token::LPar
                | Token::LBrace
                | Token::If
                | Token::While
                | Token::For
                | Token::Loop
                | Token::At
                | Token::Or
                | Token::Ident(_)
                | Token::Minus
                | Token::Bang
                | Token::Ampersand
                | Token::Star
                | Token::Return
                | Token::Break
                | Token::Continue
        ) || self.is_literal()
    }

    pub fn delim(&self) -> bool {
        self.delim_close() || self.delim_open()
    }

    pub fn delim_open(&self) -> bool {
        matches!(self, Token::LPar | Token::LBrace | Token::LBracket)
    }

    pub fn delim_close(&self) -> bool {
        matches!(self, Token::RPar | Token::RBrace | Token::RBracket)
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::CharLiteral(_)
                | Token::StringLiteral(_)
                | Token::TrueLiteral
                | Token::FalseLiteral
                | Token::NumericLiteral(_)
        )
    }

    pub fn eof(&mut self) -> bool {
        matches!(self, Token::Eof)
    }
}

fn display_token_text(f: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    write!(f, "`{}`", text.escape_debug())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_punctuation() {
        assert_eq!(Token::Semicolon.to_string(), "`;`");
        assert_eq!(format!("{:#}", Token::Semicolon), "`;`");
    }

    #[test]
    fn display_ident_and_alternate() {
        assert_eq!(Token::Ident("meow").to_string(), "`meow`");
        assert_eq!(format!("{:#}", Token::Ident("meow")), "ident");
    }

    #[test]
    fn display_literals_and_alternate() {
        assert_eq!(
            Token::StringLiteral(StringLiteral::new("a\nb")).to_string(),
            "`\\\"a\\\\nb\\\"`"
        );
        assert_eq!(format!("{:#}", Token::TrueLiteral), "literal");
    }

    #[test]
    fn display_eof() {
        assert_eq!(Token::Eof.to_string(), "end of file");
        assert_eq!(format!("{:#}", Token::Eof), "end of file");
    }
}
