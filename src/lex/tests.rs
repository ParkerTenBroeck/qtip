use super::*;

fn lex_all(input: &str) -> Result<Vec<Spanned<Token<'_>>>, Box<Spanned<LexError>>> {
    let mut lexer = Lexer::new(input);
    let mut out = Vec::new();

    loop {
        let tok = lexer.next_token()?;
        let is_eof = matches!(tok.val, Token::Eof);
        out.push(tok);
        if is_eof {
            break;
        }
    }

    Ok(out)
}

fn lex_all_tokens(input: &str) -> Result<Vec<Token<'_>>, Box<Spanned<LexError>>> {
    Ok(lex_all(input)?.into_iter().map(|t| t.val).collect())
}

fn lex_err(input: &str) -> LexError {
    match lex_all(input) {
        Ok(tokens) => panic!("expected lex error, got tokens: {tokens:?}"),
        Err(err) => err.val,
    }
}

#[test]
fn lexes_single_char_tokens() {
    let tokens = lex_all_tokens("(){}[]<>~,?;:@#").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::LPar,
            Token::RPar,
            Token::LBrace,
            Token::RBrace,
            Token::LBracket,
            Token::RBracket,
            Token::LAngle,
            Token::RAngle,
            Token::BitwiseNot,
            Token::Comma,
            Token::QuestionMark,
            Token::Semicolon,
            Token::Colon,
            Token::At,
            Token::Octothorp,
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_basic_operators() {
    let tokens = lex_all_tokens("+ - * / % = ! | & ^ .").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Percent,
            Token::Assign,
            Token::LogicalNot,
            Token::BitwiseOr,
            Token::Ampersand,
            Token::BitwiseXor,
            Token::Dot,
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_compound_operators() {
    let tokens =
        lex_all_tokens("+= ++ -= -- -> *= /= %= => == >= <= != |= || &= && ^= .. ..=")
            .unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::PlusAssign,
            Token::Inc,
            Token::MinusAssign,
            Token::Dec,
            Token::SmallRightArrow,
            Token::TimesAssign,
            Token::DivideAssign,
            Token::ModuloAssign,
            Token::BigRightArrow,
            Token::Equals,
            Token::GreaterThanEq,
            Token::LessThanEq,
            Token::NotEquals,
            Token::OrAssign,
            Token::LogicalOr,
            Token::AndAssign,
            Token::LogicalAnd,
            Token::XorAssign,
            Token::RangeExclusive,
            Token::RangeInclusive,
            Token::Eof,
        ]
    );
}

#[test]
fn skips_whitespace() {
    let tokens = lex_all_tokens(" \t\n\r  let   x \n =\t5 ").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Let,
            Token::Ident("x"),
            Token::Assign,
            Token::NumericLiteral(Number::new("5", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_keywords() {
    let tokens = lex_all_tokens(
        "true false return let for fn while loop if static as mut const break continue",
    )
    .unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::TrueLiteral,
            Token::FalseLiteral,
            Token::Return,
            Token::Let,
            Token::For,
            Token::Fn,
            Token::While,
            Token::Loop,
            Token::If,
            Token::Static,
            Token::As,
            Token::Mut,
            Token::Const,
            Token::Break,
            Token::Continue,
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_identifiers() {
    let tokens = lex_all_tokens("_ a abc hello_world x123 _tmp").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Ident("_"),
            Token::Ident("a"),
            Token::Ident("abc"),
            Token::Ident("hello_world"),
            Token::Ident("x123"),
            Token::Ident("_tmp"),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_identifier_labels() {
    let tokens = lex_all_tokens("start: loop_label:").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Label("start"),
            Token::Label("loop_label"),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_string_literal() {
    let tokens = lex_all_tokens(r#""hello""#).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::StringLiteral(StringLiteral {
                repr: "hello",
                escaped: false,
            }),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_escaped_string_literal() {
    let tokens = lex_all_tokens(r#""hello\nworld\"""#).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::StringLiteral(StringLiteral {
                repr: r#"hello\nworld\""#,
                escaped: true,
            }),
            Token::Eof,
        ]
    );

    let tokens = lex_all_tokens(r#""\"""#).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::StringLiteral(StringLiteral {
                repr: r#"\""#,
                escaped: true,
            }),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_char_literal() {
    let tokens = lex_all_tokens(r#"'x'"#).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::CharLiteral(StringLiteral {
                repr: "x",
                escaped: false,
            }),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_escaped_char_literal() {
    let tokens = lex_all_tokens(r#"'\''"#).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::CharLiteral(StringLiteral {
                repr: r#"\'"#,
                escaped: true,
            }),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_single_line_comment() {
    let tokens = lex_all_tokens("// hello\n").unwrap();
    assert_eq!(
        tokens,
        vec![Token::SingleLineComment(" hello\n"), Token::Eof,]
    );
}

#[test]
fn lexes_single_line_comment_at_eof() {
    let tokens = lex_all_tokens("// hello").unwrap();
    assert_eq!(
        tokens,
        vec![Token::SingleLineComment(" hello"), Token::Eof,]
    );
}

#[test]
fn lexes_multi_line_comment() {
    let tokens = lex_all_tokens("/* hello */").unwrap();
    assert_eq!(
        tokens,
        vec![Token::MultiLineComment(" hello "), Token::Eof,]
    );
}

#[test]
fn lexes_nested_multi_line_comment() {
    let tokens = lex_all_tokens("/* outer /* inner */ outer */").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::MultiLineComment(" outer /* inner */ outer "),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_decimal_integer() {
    let tokens = lex_all_tokens("12345").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("12345", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_integer_with_underscores() {
    let tokens = lex_all_tokens("1_234_567").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1_234_567", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_binary_octal_hex_numbers() {
    let tokens = lex_all_tokens("0b1010 0o755 0xdeadBEEF").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1010", Base::Bin, false).unwrap()),
            Token::NumericLiteral(Number::new("755", Base::Oct, false).unwrap()),
            Token::NumericLiteral(Number::new("deadBEEF", Base::Hex, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_float_numbers() {
    let tokens = lex_all_tokens("1.5 0.25 10e5 1.2e-3 7e+10").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1.5", Base::Int, true).unwrap()),
            Token::NumericLiteral(Number::new("0.25", Base::Int, true).unwrap()),
            Token::NumericLiteral(Number::new("10e5", Base::Int, true).unwrap()),
            Token::NumericLiteral(Number::new("1.2e-3", Base::Int, true).unwrap()),
            Token::NumericLiteral(Number::new("7e+10", Base::Int, true).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_number_suffixes() {
    let tokens = lex_all_tokens("123u32 1.5f32 0xffusize").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new_with_suffix("123u32", 3, Base::Int, false).unwrap()),
            Token::NumericLiteral(Number::new_with_suffix("1.5f32", 3, Base::Int, true).unwrap()),
            Token::NumericLiteral(Number::new_with_suffix("ffusize", 2, Base::Hex, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn does_not_treat_range_as_float() {
    let tokens = lex_all_tokens("1..2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1", Base::Int, false).unwrap()),
            Token::RangeExclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_int_then_inclusive_range_then_int() {
    let tokens = lex_all_tokens("1..=2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1", Base::Int, false).unwrap()),
            Token::RangeInclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_float_followed_by_dot_as_separate_token() {
    let tokens = lex_all_tokens("1.0.foo").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1.0", Base::Int, true).unwrap()),
            Token::Dot,
            Token::Ident("foo"),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_float_then_exclusive_range() {
    let tokens = lex_all_tokens("1.0..2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1.0", Base::Int, true).unwrap()),
            Token::RangeExclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_float_then_inclusive_range() {
    let tokens = lex_all_tokens("1.0..=2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1.0", Base::Int, true).unwrap()),
            Token::RangeInclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_float_with_fractional_part_not_range() {
    let tokens = lex_all_tokens("1.25").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1.25", Base::Int, true).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_zero_then_exclusive_range_then_int() {
    let tokens = lex_all_tokens("0..2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("0", Base::Int, false).unwrap()),
            Token::RangeExclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_zero_then_inclusive_range_then_int() {
    let tokens = lex_all_tokens("0..=2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("0", Base::Int, false).unwrap()),
            Token::RangeInclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_float_with_exponent_then_range() {
    let tokens = lex_all_tokens("1.5e10..2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1.5e10", Base::Int, true).unwrap()),
            Token::RangeExclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_float_with_suffix_then_range() {
    let tokens = lex_all_tokens("1.0f32..2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new_with_suffix("1.0f32", 3, Base::Int, true).unwrap()),
            Token::RangeExclusive,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Eof,
        ]
    );
}

#[test]
fn does_not_treat_dot_before_identifier_as_float() {
    let tokens = lex_all_tokens("1.foo").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::NumericLiteral(Number::new("1", Base::Int, false).unwrap()),
            Token::Dot,
            Token::Ident("foo"),
            Token::Eof,
        ]
    );
}

#[test]
fn tracks_spans_for_simple_tokens() {
    let toks = lex_all("let x").unwrap();
    assert_eq!(toks.len(), 3);

    assert_eq!(toks[0].val, Token::Let);
    assert_eq!(toks[0].span, Span::new(0, 3));

    assert_eq!(toks[1].val, Token::Ident("x"));
    assert_eq!(toks[1].span, Span::new(4, 5));

    assert_eq!(toks[2].val, Token::Eof);
    assert_eq!(toks[2].span, Span::new(5, 5));
}

#[test]
fn errors_on_invalid_char() {
    assert_eq!(lex_err("`"), LexError::InvalidChar('`'));
}

#[test]
fn errors_on_unclosed_string_literal() {
    assert_eq!(lex_err("\"abc"), LexError::UnclosedStringLiteral);
}

#[test]
fn errors_on_unclosed_char_literal() {
    assert_eq!(lex_err("'a"), LexError::UnclosedCharLiteral);
}

#[test]
fn errors_on_unclosed_multi_line_comment() {
    assert_eq!(lex_err("/* hello"), LexError::UnclosedMultiLineComment);
}

#[test]
fn errors_on_empty_exponent() {
    assert_eq!(lex_err("1e"), LexError::EmptyExponent);
    assert_eq!(lex_err("1e+"), LexError::EmptyExponent);
    assert_eq!(lex_err("1e_"), LexError::EmptyExponent);
}

#[test]
fn lexes_mixed_snippet() {
    let tokens = lex_all_tokens(
        r#"
            fn add(x: i32, y: i32) -> i32 {
                let z = x + y * 2;
                return z;
            }
            "#,
    )
    .unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Fn,
            Token::Ident("add"),
            Token::LPar,
            Token::Label("x"),
            Token::Ident("i32"),
            Token::Comma,
            Token::Label("y"),
            Token::Ident("i32"),
            Token::RPar,
            Token::SmallRightArrow,
            Token::Ident("i32"),
            Token::LBrace,
            Token::Let,
            Token::Ident("z"),
            Token::Assign,
            Token::Ident("x"),
            Token::Plus,
            Token::Ident("y"),
            Token::Star,
            Token::NumericLiteral(Number::new("2", Base::Int, false).unwrap()),
            Token::Semicolon,
            Token::Return,
            Token::Ident("z"),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ]
    );
}
