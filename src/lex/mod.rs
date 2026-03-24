pub mod number;
#[cfg(test)]
pub mod tests;
pub mod token;

pub use number::*;
use std::str::Chars;
pub use token::*;

use crate::span::{Span, Spanned};

type LexerResult<'a> = Result<Spanned<Token<'a>>, Box<Spanned<LexError>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexError {
    InvalidChar(char),
    UnclosedCharLiteral,
    UnclosedMultiLineComment,
    UnclosedStringLiteral,
    EmptyExponent,
    NoNumberAfterBasePrefix,
    NumberParseError(NumberError),
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::InvalidChar(char) => write!(f, "invalid character {char:?}"),
            LexError::UnclosedCharLiteral => write!(f, "unclosed char literal"),
            LexError::UnclosedMultiLineComment => write!(f, "unclosed multi line comment"),
            LexError::UnclosedStringLiteral => write!(f, "unclosed string literal"),
            LexError::EmptyExponent => write!(f, "empty exponent"),
            LexError::NoNumberAfterBasePrefix => write!(f, "no number after base prefix"),
            LexError::NumberParseError(err) => write!(f, "{err}"),
        }
    }
}

pub struct Lexer<'a> {
    str: &'a str,
    chars: Chars<'a>,
    current: usize,
}

fn ident(ident: &str) -> Token<'_> {
    match ident {
        "true" => Token::TrueLiteral,
        "false" => Token::FalseLiteral,
        "return" => Token::Return,
        "let" => Token::Let,
        "for" => Token::For,
        "fn" => Token::Fn,
        "while" => Token::While,
        "loop" => Token::Loop,
        "if" => Token::If,
        "static" => Token::Static,
        "as" => Token::As,
        "mut" => Token::Mut,
        "const" => Token::Const,
        "break" => Token::Break,
        "continue" => Token::Continue,
        o => Token::Ident(o),
    }
}

fn ident_start(char: char) -> bool {
    char == '_' || char.is_alphabetic()
}

fn ident_continue(char: char) -> bool {
    char == '_' || char.is_alphanumeric()
}

impl<'a> Lexer<'a> {
    pub fn new(str: &'a str) -> Self {
        Self {
            str,
            chars: str.chars(),
            current: 0,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.current += c.len_utf8();
        Some(c)
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.clone().next()
    }

    fn peek_peek_char(&mut self) -> Option<char> {
        self.chars.clone().nth(1)
    }

    pub fn next_token(&mut self) -> LexerResult<'a> {
        macro_rules! consume_if {
            ($pat:pat) => {{
                match self.peek_char() {
                    Some(value @ $pat) => {
                        self.next_char();
                        Some(value)
                    }
                    value => value,
                }
            }};
        }

        loop {
            let start = self.current;

            let tok = match self.next_char() {
                Some('+') => match consume_if!('=' | '+') {
                    Some('=') => Ok(Token::PlusAssign),
                    Some('+') => Ok(Token::Inc),
                    _ => Ok(Token::Plus),
                },
                Some('-') => match consume_if!('>' | '=' | '-') {
                    Some('>') => Ok(Token::SmallRightArrow),
                    Some('=') => Ok(Token::MinusAssign),
                    Some('-') => Ok(Token::Dec),
                    _ => Ok(Token::Minus),
                },
                Some('*') => match consume_if!('=') {
                    Some('=') => Ok(Token::TimesAssign),
                    _ => Ok(Token::Star),
                },
                Some('/') => match consume_if!('=' | '/' | '*') {
                    Some('=') => Ok(Token::DivideAssign),
                    Some('/') => loop {
                        if matches!(self.next_char(), None | Some('\n')) {
                            break Ok(Token::SingleLineComment(&self.str[start + 2..self.current]));
                        }
                    },
                    Some('*') => {
                        let mut count = 0usize;
                        loop {
                            match (self.next_char(), self.peek_char()) {
                                (Some('/'), Some('*')) => count += 1,
                                (Some('*'), Some('/')) if count > 0 => count -= 1,
                                (Some('*'), Some('/')) => {
                                    self.next_char();
                                    break Ok(Token::MultiLineComment(
                                        &self.str[start + 2..self.current - 2],
                                    ));
                                }
                                (None, _) => break Err(LexError::UnclosedMultiLineComment),
                                _ => {}
                            }
                        }
                    }
                    _ => Ok(Token::Slash),
                },
                Some('%') => match consume_if!('=') {
                    Some('=') => Ok(Token::ModuloAssign),
                    _ => Ok(Token::Percent),
                },
                Some('=') => match consume_if!('>' | '=') {
                    Some('>') => Ok(Token::BigRightArrow),
                    Some('=') => Ok(Token::Equals),
                    _ => Ok(Token::Assign),
                },
                Some('>') => match consume_if!('=' | '>') {
                    Some('=') => Ok(Token::GreaterThanEq),
                    Some('>') => match consume_if!('=') {
                        Some('=') => Ok(Token::ShiftRightAssign),
                        _ => Ok(Token::ShiftRight),
                    },
                    _ => Ok(Token::GreaterThan),
                },
                Some('<') => match consume_if!('=' | '<') {
                    Some('=') => Ok(Token::LessThanEq),
                    Some('<') => match consume_if!('=') {
                        Some('=') => Ok(Token::ShiftLeftAssign),
                        _ => Ok(Token::ShiftLeft),
                    },
                    _ => Ok(Token::LessThan),
                },
                Some('!') => match consume_if!('=') {
                    Some('=') => Ok(Token::NotEquals),
                    _ => Ok(Token::LogicalNot),
                },
                Some('|') => match consume_if!('=' | '|') {
                    Some('=') => Ok(Token::OrAssign),
                    Some('|') => Ok(Token::LogicalOr),
                    _ => Ok(Token::BitwiseOr),
                },
                Some('&') => match consume_if!('=' | '&') {
                    Some('=') => Ok(Token::AndAssign),
                    Some('&') => Ok(Token::LogicalAnd),
                    _ => Ok(Token::Ampersand),
                },
                Some('^') => match consume_if!('=') {
                    Some('=') => Ok(Token::XorAssign),
                    _ => Ok(Token::BitwiseXor),
                },
                Some('.') => match consume_if!('.') {
                    Some('.') => match consume_if!('=') {
                        Some('=') => Ok(Token::RangeInclusive),
                        _ => Ok(Token::RangeExclusive),
                    },
                    _ => Ok(Token::Dot),
                },

                Some(kind @ ('\'' | '"')) => {
                    let mut escaped = false;
                    loop {
                        match self.next_char() {
                            Some('\\') => {
                                if self.next_char().is_some() {
                                    escaped = true
                                }
                            }
                            Some(c) if c == kind => {
                                let repr = &self.str[start + 1..self.current - 1];
                                if kind == '"' {
                                    break Ok(Token::StringLiteral(StringLiteral {
                                        repr,
                                        escaped,
                                    }));
                                } else {
                                    break Ok(Token::CharLiteral(StringLiteral { repr, escaped }));
                                }
                            }
                            Some(_) => {}

                            None if kind == '"' => break Err(LexError::UnclosedStringLiteral),
                            None => break Err(LexError::UnclosedCharLiteral),
                        }
                    }
                }
                Some('(') => Ok(Token::LPar),
                Some(')') => Ok(Token::RPar),
                Some('{') => Ok(Token::LBrace),
                Some('}') => Ok(Token::RBrace),
                Some('[') => Ok(Token::LBracket),
                Some(']') => Ok(Token::RBracket),
                Some('~') => Ok(Token::BitwiseNot),
                Some(',') => Ok(Token::Comma),
                Some('?') => Ok(Token::QuestionMark),
                Some(';') => Ok(Token::Semicolon),
                Some(':') => Ok(Token::Colon),
                Some('@') => Ok(Token::At),
                Some('$') => Ok(Token::Ampersand),
                Some('#') => Ok(Token::Octothorp),

                Some('0') => self.lex_number(start, true),
                Some('1'..='9') => self.lex_number(start, false),

                Some(c) if c.is_whitespace() => continue,
                Some(c) if ident_start(c) => loop {
                    match self.peek_char() {
                        Some(c) if ident_continue(c) => _ = self.next_char(),
                        Some(':') => {
                            self.next_char();
                            break Ok(Token::Label(&self.str[start..self.current - 1]));
                        }
                        _ => break Ok(ident(&self.str[start..self.current])),
                    }
                },
                Some(c) => Err(LexError::InvalidChar(c)),
                None => Ok(Token::Eof),
            };

            match tok {
                Ok(token) => {
                    let meta = Span::new(start, self.current);
                    return Ok(Spanned::new(token, meta));
                }
                Err(err) => {
                    let meta = Span::new(start, self.current);
                    return Err(Box::new(Spanned::new(err, meta)));
                }
            }
        }
    }

    fn lex_number(&mut self, start: usize, started_with_zero: bool) -> Result<Token<'a>, LexError> {
        let mut base = Base::Int;
        let mut float = false;
        let mut numeric_start = start;

        if started_with_zero {
            match self.peek_char() {
                Some('b') => {
                    self.next_char();
                    numeric_start = self.current;
                    base = Base::Bin;
                }
                Some('o') => {
                    self.next_char();
                    numeric_start = self.current;
                    base = Base::Oct;
                }
                Some('x') => {
                    self.next_char();
                    numeric_start = self.current;
                    base = Base::Hex;
                }
                _ => {}
            }
        }

        loop {
            match self.peek_char() {
                Some('0'..='9') => _ = self.next_char(),
                Some('a'..='f' | 'A'..='F') if base == Base::Hex => _ = self.next_char(),
                Some('.') if !float => {
                    let second = self.peek_peek_char().unwrap_or('\0');
                    if ident_start(second) || second == '.' {
                        break;
                    } else {
                        float = true;
                        self.next_char();
                    }
                }
                Some('_') => _ = self.next_char(),
                _ => break,
            }
        }

        if matches!(self.peek_char(), Some('e')) {
            self.next_char();
            float = true;

            while matches!(self.peek_char(), Some('_')) {
                self.next_char();
            }

            if matches!(self.peek_char(), Some('+' | '-')) {
                self.next_char();
            }

            while matches!(self.peek_char(), Some('_')) {
                self.next_char();
            }

            let exp_start = self.current;
            while matches!(self.peek_char(), Some('0'..='9' | '_')) {
                self.next_char();
            }

            if exp_start == self.current {
                return Err(LexError::EmptyExponent);
            }
        }

        match self.peek_char() {
            Some('a'..='z' | 'A'..='Z') => {
                let suffix_start = self.current;
                self.next_char();

                while matches!(
                    self.peek_char(),
                    Some('0'..='9' | 'a'..='z' | 'A'..='Z' | '_')
                ) {
                    self.next_char();
                }

                if matches!(self.peek_char(), Some(':')) {
                    self.next_char();
                }

                let len = suffix_start - numeric_start;
                Number::new_with_suffix(&self.str[numeric_start..self.current], len, base, float)
                    .map(Token::NumericLiteral)
                    .map_err(LexError::NumberParseError)
            }
            _ => Number::new(&self.str[numeric_start..self.current], base, float)
                .map(Token::NumericLiteral)
                .map_err(LexError::NumberParseError),
        }
    }
}
