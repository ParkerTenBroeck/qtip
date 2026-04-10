use crate::parser::*;

use crate::{diag::parse::*, lex::Token};

impl<'a> Parser<'a> {
    fn parse_type_list(&mut self) -> PResult<(Vec<ast::Type<'a>>, bool)> {
        let mut params = vec![];

        let mut trailing_comma = false;

        let params = self.parse_delim(Delimiter::Paren, |parser| {
            while !matches!(parser.next.value, Token::RPar | Token::Eof) {
                match parser.parse_type() {
                    Ok(ok) => params.push(ok),
                    Err(_) => {
                        while !matches!(parser.next.value, Token::RPar | Token::Eof | Token::Comma)
                        {
                            parser.next();
                        }
                    }
                }
                trailing_comma = parser.consume_if(Token::Comma);
                if !trailing_comma {
                    break;
                }
            }
            params
        })?;

        Ok((params, trailing_comma))
    }

    pub(super) fn parse_type(&mut self) -> PResult<ast::Type<'a>> {

        let kind = match self.next.value {
            Token::Ident(_) | Token::Colon => ast::TypeKind::Path(self.parse_path()?),
            Token::Bang => {
                self.next();
                ast::TypeKind::Never
            }
            Token::Ampersand => {
                self.next();
                let ref_kind = match self.next.value {
                    Token::Ident("raw") => {
                        self.next();
                        ast::RefKind::Ptr
                    }
                    Token::Ident("pin") => {
                        self.next();
                        ast::RefKind::Pinned
                    }
                    _ => ast::RefKind::Ref,
                };
                let mutability = if self.consume_if(Token::Mut) {
                    ast::Mutability::Mut
                } else {
                    ast::Mutability::Const
                };
                ast::TypeKind::Ref(mutability, ref_kind, Box::new(self.parse_type()?))
            }
            Token::Star => {
                self.next();
                let mutability = if self.consume_if(Token::Const) {
                    ast::Mutability::Const
                } else if self.consume_if(Token::Mut) {
                    ast::Mutability::Mut
                } else {
                    self.ctx.report(ExpectedType {
                        node: self.previous.node,
                        found: self.previous.value,
                    });
                    return Err(());
                };
                ast::TypeKind::Ref(mutability, ast::RefKind::Ptr, Box::new(self.parse_type()?))
            }
            Token::Fn => {
                self.next();
                if !self.next.value.delim_open() {
                    self.ctx.report(MissingDelimiters {
                        node: self.previous.node.after(),
                        suggestion: "missing parameters for fn type",
                        missing: "add a parameter list",
                        delims: "()",
                    });
                    return Err(());
                }
                let args = self.parse_type_list()?;
                let ret = if self.consume_if(Token::SmallRightArrow) {
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                ast::TypeKind::FnPtr(args.0, ret)
            }
            Token::LPar => {
                let (mut types, trailing_comma) = self.parse_type_list()?;
                if types.len() == 1 && !trailing_comma {
                    ast::TypeKind::Paren(Box::new(types.remove(0)))
                } else {
                    ast::TypeKind::Tuple(types)
                }
            }
            Token::LBracket => {
                self.next();
                let ty = self.parse_type()?;

                if self.consume_if(Token::Semicolon) {
                    let expr = self.parse_expr()?;
                    self.expect_token(Token::RBracket)?;
                    ast::TypeKind::Array(Box::new(ty), Box::new(expr))
                } else {
                    self.expect_token(Token::RBracket)?;
                    ast::TypeKind::Slice(Box::new(ty))
                }
            }
            _ => {
                self.ctx.report(ExpectedType {
                    node: self.next.node,
                    found: self.next.value,
                });
                if !self.next.value.delim() {
                    self.next();
                }
                return Err(());
            }
        };
        Ok(ast::Type { kind })
    }
}
