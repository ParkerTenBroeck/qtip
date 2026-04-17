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
            Token::Union => ast::ItemKind::Union(self.parse_union()?),
            Token::Struct => ast::ItemKind::Struct(self.parse_struct()?),
            Token::Enum => ast::ItemKind::Enum(self.parse_enum()?),
            Token::Static => ast::ItemKind::Static(self.parse_static()?),
            Token::Const => ast::ItemKind::Constant(self.parse_const()?),
            Token::Mod => ast::ItemKind::Module(self.parse_mod()?),
            Token::Use => ast::ItemKind::Use(self.parse_use()?),
            Token::Fn => ast::ItemKind::Fn(self.parse_fn()?),
            Token::Ident("extern") => ast::ItemKind::Extern(self.parse_extern_items()?),
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

    fn parse_use(&mut self) -> PResult<ast::Use<'a>> {
        let use_tree = self.parse_use_tree()?;
        self.expect_semi();
        Ok(use_tree)
    }

    fn parse_use_tree(&mut self) -> PResult<ast::Use<'a>> {
        let start = self.next.node;
        let kind = match self.next.value {
            Token::Star => {
                self.next();
                ast::UseKind::Star(self.previous.node)
            }
            _ => ast::UseKind::Sym(self.parse_symbol()?),
        };

        let mut childern = vec![];
        if self.next.value == Token::Colon {
            self.consume_path_sep();

            if self.next.value == Token::LBrace {
                self.parse_delim(Delimiter::Brace, |parser| {
                    while !matches!(parser.next.value, Token::RBrace | Token::Eof) {
                        match parser.parse_use_tree() {
                            Ok(child) => childern.push(child),
                            Err(_) => {
                                while !matches!(
                                    parser.next.value,
                                    Token::RBrace | Token::Eof | Token::Comma
                                ) {
                                    parser.next();
                                }
                            }
                        }

                        if !parser.consume_if(Token::Comma) {
                            break;
                        }
                    }
                })?;
            } else {
                childern.push(self.parse_use_tree()?);
            }
        }

        Ok(ast::Use {
            node: self.ctx.join(start, self.previous.node),
            kind,
            childern,
        })
    }

    fn parse_named_field(&mut self) -> PResult<ast::NammedField<'a>> {
        let name = self.parse_symbol()?;
        self.expect_token(Token::Colon)?;
        let ty = self.parse_type()?;
        Ok(ast::NammedField { name, ty })
    }

    fn parse_named_field_list(&mut self) -> PResult<PResult<Vec<ast::NammedField<'a>>>> {
        let mut fields = vec![];

        self.parse_delim(Delimiter::Brace, |parser| {
            while !matches!(parser.next.value, Token::RBrace | Token::Eof) {
                match parser.parse_named_field() {
                    Ok(field) => fields.push(field),
                    Err(_) => {
                        while !matches!(
                            parser.next.value,
                            Token::RBrace | Token::Eof | Token::Comma
                        ) {
                            parser.next();
                        }
                    }
                }

                if !parser.consume_if(Token::Comma) {
                    break;
                }
            }

            Ok(fields)
        })
    }

    fn parse_fields_kind(&mut self) -> PResult<ast::FieldsKind<'a>> {
        match self.next.value {
            Token::LBrace => Ok(ast::FieldsKind::Nammed(
                self.parse_named_field_list()?.unwrap_or_default(),
            )),
            Token::LPar => {
                let (fields, _) = self.parse_type_list()?;
                self.expect_semi();
                Ok(ast::FieldsKind::Tuple(fields))
            }
            Token::Semicolon => {
                self.next();
                Ok(ast::FieldsKind::None)
            }
            _ => {
                self.ctx.report(UnexpectedToken {
                    node: self.previous.node.after(),
                    found: self.next.value,
                    expected: Token::LBrace,
                });
                Err(())
            }
        }
    }

    fn parse_struct(&mut self) -> PResult<ast::Struct<'a>> {
        let name = self.parse_symbol()?;
        let fields = self.parse_fields_kind()?;
        Ok(ast::Struct { name, fields })
    }

    fn parse_union(&mut self) -> PResult<ast::Union<'a>> {
        let name = self.parse_symbol()?;
        let fields = self.parse_named_field_list()?.unwrap_or_default();
        Ok(ast::Union { name, fields })
    }

    fn parse_enum(&mut self) -> PResult<ast::Enum<'a>> {
        let name = self.parse_symbol()?;
        let mut varients = vec![];
        self.parse_delim(Delimiter::Brace, |parser| {
            while !matches!(parser.next.value, Token::RBrace | Token::Eof) {
                match parser.parse_enum_varient() {
                    Ok(varient) => varients.push(varient),
                    Err(_) => {
                        while !matches!(
                            parser.next.value,
                            Token::RBrace | Token::Eof | Token::Comma
                        ) {
                            parser.next();
                        }
                    }
                }

                if !parser.consume_if(Token::Comma) {
                    break;
                }
            }
        })?;
        Ok(ast::Enum { name, varients })
    }

    fn parse_enum_varient(&mut self) -> PResult<ast::EnumVarient<'a>> {
        let name = self.parse_symbol()?;
        let fields = match self.next.value {
            Token::LBrace => {
                ast::FieldsKind::Nammed(self.parse_named_field_list()?.unwrap_or_default())
            }
            Token::LPar => {
                let (fields, _) = self.parse_type_list()?;
                ast::FieldsKind::Tuple(fields)
            }
            _ => ast::FieldsKind::None,
        };

        Ok(ast::EnumVarient { name, fields })
    }

    fn parse_static(&mut self) -> PResult<ast::StaticItem<'a>> {
        let mutability = if self.consume_if(Token::Mut) {
            ast::Mutability::Mut
        } else {
            ast::Mutability::Const
        };
        let name = self.parse_symbol()?;
        self.expect_token(Token::Colon)?;
        let ty = self.parse_type()?;
        let expr = if self.consume_if(Token::Assign) {
            Some(self.parse_expr(true)?)
        } else {
            None
        };
        self.expect_semi();

        Ok(ast::StaticItem {
            name,
            ty,
            mutability,
            expr,
        })
    }

    fn parse_const(&mut self) -> PResult<ast::ConstItem<'a>> {
        let mutability = if self.consume_if(Token::Mut) {
            ast::Mutability::Mut
        } else {
            ast::Mutability::Const
        };
        let name = self.parse_symbol()?;
        self.expect_token(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect_token(Token::Assign)?;
        let expr = self.parse_expr(true)?;
        self.expect_semi();

        Ok(ast::ConstItem {
            name,
            ty,
            mutability,
            expr,
        })
    }

    fn parse_extern_items(&mut self) -> PResult<Vec<ast::Item<'a>>> {
        if matches!(
            self.next.value,
            Token::StringLiteral(_) | Token::CharLiteral(_) | Token::Ident(_)
        ) {
            self.next();
        }

        let mut items = vec![];
        self.parse_delim(Delimiter::Brace, |parser| {
            while !matches!(parser.next.value, Token::RBrace | Token::Eof) {
                let level = parser.delimiter_stack.len();

                match parser.parse_item() {
                    Ok(item) => items.push(item),
                    Err(_) => {
                        while !parser.next.value.eof()
                            && (!parser.next.value.starts_item()
                                || level < parser.delimiter_stack.len())
                        {
                            parser.next();
                        }
                    }
                }
            }
        })?;
        Ok(items)
    }

    fn parse_fn_params(&mut self) -> PResult<PResult<Vec<ast::FnParam<'a>>>> {
        let mut params = vec![];

        if !self.next.value.delim_open() {
            self.ctx.report(MissingFnParamList {
                node: self.previous.node.after(),
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
                self.ctx.report(MissingFnBody {
                    node: self.previous.node.after(),
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
