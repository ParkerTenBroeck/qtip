use crate::parser::ast::Symbol;
use crate::parser::*;

use crate::{diag::parse::*, lex::Token};

impl<'a> Parser<'a> {
    pub(super) fn parse_type(&mut self) -> PResult<ast::Type<'a>> {
        match self.next.value {
            Token::Ident(name) => {
                self.next();
                Ok(ast::Type {
                    name: Symbol {
                        name,
                        node: self.previous.node,
                    },
                })
            }
            _ => {
                self.ctx.report(ExpectedType {
                    node: self.next.node,
                    found: self.next.value,
                });
                if !self.next.value.delim() {
                    self.next();
                }
                Err(())
            }
        }
    }
}
