use crate::parser::*;

use crate::{diag::parse::*, lex::Token};

impl<'a> Parser<'a> {
    pub(super) fn parse_item_list(&mut self) -> Vec<ast::Item<'a>> {
        let mut list = vec![];

        while self.next.value != Token::Eof {
            let level = self.delimiter_stack.len();

            match self.parse_item() {
                Ok(item) => list.push(item),
                Err(_) => {
                    // tries to recover by ignoring the remainder of the invalid item
                    while !self.next.value.eof()
                        && (!self.next.value.starts_item() || level < self.delimiter_stack.len())
                    {
                        self.next();
                    }
                }
            }
        }
        self.next();

        list
    }

    pub(super) fn parse_item(&mut self) -> PResult<ast::Item<'a>> {
        let start = self.next.node;

        let vis = self.parse_vis();
        let kind = match self.next().value {
            Token::Union => todo!(),
            Token::Struct => todo!(),
            Token::Enum => todo!(),
            Token::Static => todo!(),
            Token::Const => todo!(),
            Token::Mod => ast::ItemKind::Module(self.parse_mod()?),
            Token::Fn => ast::ItemKind::Fn(self.parse_fn()?),
            _ => {
                self.ctx.report(ExpectedItem {
                    node: self.previous.node,
                    found: self.previous.value,
                    remove_semi: (self.previous.value == Token::Semicolon)
                        .then_some(self.previous.node),
                });
                return Err(());
            }
        };
        Ok(ast::Item {
            node: self.ctx.join(start, self.previous.node),
            kind,
            vis,
        })
    }

    fn parse_mod(&mut self) -> PResult<ast::Module<'a>> {
        let name = self.parse_symbol()?;
        self.expect_semi();
        Ok(ast::Module { name })
    }

    fn parse_fn_params(&mut self) -> PResult<PResult<Vec<ast::FnParam<'a>>>> {
        let mut params = vec![];

        if !self.next.value.delim_open() {
            self.ctx.report(MissingDelimiters {
                node: self.previous.node.after(),
                suggestion: "missing parameters for function definition",
                missing: "add a parameter list",
                delims: "()",
            });
            return Err(());
        }

        self.parse_delim(Delimiter::Paren, move |parser| {
            while !matches!(parser.next.value, Token::RPar | Token::Eof) {
                match parser.parse_fn_param() {
                    Ok(ok) => params.push(ok),
                    Err(_) => {
                        while !matches!(parser.next.value, Token::RPar | Token::Eof | Token::Comma)
                        {
                            parser.next();
                        }
                    }
                }
                if !parser.consume_if(Token::Comma) {
                    break;
                }
            }
            Ok(params)
        })
    }

    fn parse_fn(&mut self) -> PResult<ast::Fn<'a>> {
        let name = self.parse_symbol()?;
        let params = self.parse_fn_params()?.unwrap_or_default();
        let ret = if self.consume_if(Token::SmallRightArrow) {
            self.parse_type().ok()
        } else {
            None
        };
        let body = if self.consume_if(Token::Semicolon) {
            None
        } else {
            if !self.next.value.delim_open() {
                self.ctx.report(MissingDelimiters {
                    node: self.previous.node.after(),
                    suggestion: "function body",
                    missing: "function body",
                    delims: "{}",
                });
                return Err(());
            }
            self.parse_block()?.ok()
        };

        Ok(ast::Fn {
            name,
            params,
            ret,
            body,
        })
    }
}
