use crate::parser::*;

use crate::{diag::parse::*, lex::Token};

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self, allow_struct_init: bool) -> PResult<ast::Expr<'a>> {
        self.parse_expr_binop(0, allow_struct_init)
    }

    fn parse_expr_binop(&mut self, min_prec: u32, allow_struct_init: bool) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;

        let mut lhs = self.parse_expr_2(allow_struct_init)?;

        use ast::BinOp;
        loop {
            let op = match self.next.value {
                Token::Plus if BinOp::Add.precedence() >= min_prec => BinOp::Add,
                Token::Minus if BinOp::Sub.precedence() >= min_prec => BinOp::Sub,
                Token::Star if BinOp::Mul.precedence() >= min_prec => BinOp::Mul,
                Token::Slash if BinOp::Div.precedence() >= min_prec => BinOp::Div,
                Token::Percent if BinOp::Rem.precedence() >= min_prec => BinOp::Rem,
                Token::OrOr if BinOp::Or.precedence() >= min_prec => BinOp::Or,
                Token::Or if BinOp::Or.precedence() >= min_prec => BinOp::Or,
                Token::Ampersand if BinOp::And.precedence() >= min_prec => BinOp::And,
                Token::AndAnd if BinOp::And.precedence() >= min_prec => BinOp::And,
                Token::Carrot if BinOp::Xor.precedence() >= min_prec => BinOp::Xor,
                Token::ShiftLeft if BinOp::Shl.precedence() >= min_prec => BinOp::Shl,
                Token::ShiftRight if BinOp::Shr.precedence() >= min_prec => BinOp::Shr,
                Token::GreaterThan if BinOp::Gt.precedence() >= min_prec => {
                    self.next();
                    BinOp::Gt
                }
                Token::LessThan if BinOp::Lt.precedence() >= min_prec => {
                    self.next();
                    BinOp::Lt
                }

                Token::GreaterThanEq if BinOp::Gte.precedence() >= min_prec => BinOp::Gte,
                Token::LessThanEq if BinOp::Lte.precedence() >= min_prec => BinOp::Lte,
                Token::Equals if BinOp::Eq.precedence() >= min_prec => BinOp::Eq,
                Token::NotEquals if BinOp::Ne.precedence() >= min_prec => BinOp::Ne,

                Token::Assign if BinOp::Assign.precedence() >= min_prec => BinOp::Assign,
                Token::PlusAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                    BinOp::PlusAssign
                }
                Token::MinusAssign if BinOp::MinusAssign.precedence() >= min_prec => {
                    BinOp::MinusAssign
                }
                Token::TimesAssign if BinOp::TimesAssign.precedence() >= min_prec => {
                    BinOp::TimesAssign
                }
                Token::DivideAssign if BinOp::DivideAssign.precedence() >= min_prec => {
                    BinOp::DivideAssign
                }
                Token::ModuloAssign if BinOp::ModuloAssign.precedence() >= min_prec => {
                    BinOp::ModuloAssign
                }
                Token::OrAssign if BinOp::OrAssign.precedence() >= min_prec => BinOp::OrAssign,
                Token::AndAssign if BinOp::AndAssign.precedence() >= min_prec => BinOp::AndAssign,
                Token::XorAssign if BinOp::XorAssign.precedence() >= min_prec => BinOp::XorAssign,
                // Token::ShiftRightAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                //     BinOp::PlusAssign
                // }
                // Token::ShiftLeftAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                //     BinOp::PlusAssign
                // }
                _ => break,
            };
            self.next();

            let rhs = self.parse_expr_binop(op.precedence() + op.right_to_left() as u32, allow_struct_init)?;
            lhs = ast::Expr {
                node: self.ctx.join(start, self.previous.node),
                kind: ast::ExprKind::BinOp {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                },
            }
        }

        Ok(lhs)
    }

    fn parse_expr_2(&mut self, allow_struct_init: bool) -> PResult<ast::Expr<'a>> {
        //todo parse as
        self.parse_expr_3(allow_struct_init)
    }

    fn parse_expr_3(&mut self, allow_struct_init: bool) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;
        let kind = match self.next.value {
            Token::Minus => {
                self.next();
                ast::ExprKind::UnOp {
                    expr: Box::new(self.parse_expr_3(allow_struct_init)?),
                    op: ast::UnOp::Neg,
                }
            }
            Token::Bang => {
                self.next();
                ast::ExprKind::UnOp {
                    expr: Box::new(self.parse_expr_3(allow_struct_init)?),
                    op: ast::UnOp::Not,
                }
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
                ast::ExprKind::Ref(mutability, ref_kind, Box::new(self.parse_expr_3(allow_struct_init)?))
            }
            Token::Star => {
                self.next();
                ast::ExprKind::Deref(Box::new(self.parse_expr_3(allow_struct_init)?))
            }
            _ => return self.parse_expr_postfix(allow_struct_init),
        };

        Ok(ast::Expr {
            node: self.ctx.join(start, self.previous.node),
            kind,
        })
    }

    fn parse_expr_postfix(&mut self, allow_struct_init: bool) -> PResult<ast::Expr<'a>> {
        let mut expr = self.parse_expr_bottom(allow_struct_init)?;

        loop {
            expr = match self.next.value {
                Token::LPar => {
                    let start = expr.node;
                    let mut args = vec![];
                    self.parse_delim(Delimiter::Paren, |parser| {
                        while !matches!(parser.next.value, Token::RPar | Token::Eof) {
                            match parser.parse_expr(true) {
                                Ok(arg) => args.push(arg),
                                Err(_) => {
                                    while !matches!(
                                        parser.next.value,
                                        Token::RPar | Token::Eof | Token::Comma
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

                    ast::Expr {
                        node: self.ctx.join(start, self.previous.node),
                        kind: ast::ExprKind::FuncCall {
                            ptr: Box::new(expr),
                            args,
                        },
                    }
                }
                Token::Dot => {
                    let start = expr.node;
                    self.next();
                    let field = self.parse_symbol()?;
                    ast::Expr {
                        node: self.ctx.join(start, self.previous.node),
                        kind: ast::ExprKind::Field(Box::new(expr), field),
                    }
                }
                Token::LBracket => {
                    let start = expr.node;
                    let index =
                        self.parse_delim(Delimiter::Bracket, |parser| parser.parse_expr(true))??;
                    ast::Expr {
                        node: self.ctx.join(start, self.previous.node),
                        kind: ast::ExprKind::Index(Box::new(expr), Box::new(index)),
                    }
                }
                _ => break,
            };
        }

        Ok(expr)
    }

    fn parse_if_chain(&mut self, label: Option<ast::Label<'a>>) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;

        let kind = if self.consume_if(Token::If) {
            let cond = self.parse_expr(false)?;
            let block = self.parse_block()?;
            let chain = if self.consume_if(Token::Else) {
                Some(Box::new(self.parse_if_chain(None)?))
            } else {
                None
            };
            ast::ExprKind::If(Box::new(cond), block?, chain, label)
        } else {
            ast::ExprKind::Block(self.parse_block()??, None)
        };

        Ok(ast::Expr {
            node: self.ctx.join(start, self.previous.node),
            kind,
        })
    }

    fn parse_expr_labled(&mut self, label: Option<ast::Label<'a>>) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;
        let kind = match self.next.value {
            Token::If => return self.parse_if_chain(label),
            Token::While => {
                ast::ExprKind::While(
                    Box::new(self.parse_expr(false)?),
                    self.parse_block()??,
                    label,
                )
            }
            Token::LBrace => ast::ExprKind::Block(self.parse_block()??, label),
            Token::Loop => ast::ExprKind::Loop(self.parse_block()??, label),
            Token::For => ast::ExprKind::For(self.parse_block()??, label),
            _ => {
                self.ctx.report(ExpectedLabeledExpression {
                    node: self.next.node,
                    found: self.next.value,
                });
                self.next();
                return Err(());
            }
        };

        Ok(ast::Expr {
            node: self.ctx.join(start, self.previous.node),
            kind,
        })
    }

    fn parse_label_opt(&mut self) -> PResult<Option<ast::Label<'a>>> {
        if self.consume_if(Token::At) {
            Ok(Some(ast::Label {
                sym: self.parse_symbol()?,
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_expr_bottom(&mut self, allow_struct_init: bool) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;
        let kind = match self.next.value {
            Token::LBrace | Token::If | Token::While | Token::Loop | Token::For => {
                return self.parse_expr_labled(None);
            }
            Token::Return => {
                self.next();
                let expr = if self.next.value.starts_stmt() {
                    Some(Box::new(self.parse_expr(true)?))
                } else {
                    None
                };
                ast::ExprKind::Return(expr)
            }
            Token::Continue => {
                self.next();
                let label = self.parse_label_opt()?;
                ast::ExprKind::Continue(label)
            }
            Token::Break => {
                self.next();
                let label = self.parse_label_opt()?;
                let expr = if self.next.value.starts_stmt() {
                    Some(Box::new(self.parse_expr(true)?))
                } else {
                    None
                };
                ast::ExprKind::Break(expr, label)
            }
            Token::At => {
                let label = self.parse_label_opt()?;
                return self.parse_expr_labled(label);
            }
            Token::CharLiteral(c) => {
                self.next();
                ast::ExprKind::Literal(ast::Literal::Char(c))
            }
            Token::StringLiteral(c) => {
                self.next();
                ast::ExprKind::Literal(ast::Literal::String(c))
            }
            Token::NumericLiteral(c) => {
                self.next();
                ast::ExprKind::Literal(ast::Literal::Number(c))
            }
            Token::FalseLiteral => {
                self.next();
                ast::ExprKind::Literal(ast::Literal::Bool(false))
            }
            Token::TrueLiteral => {
                self.next();
                ast::ExprKind::Literal(ast::Literal::Bool(true))
            }
            Token::Or => return self.parse_lambda(),
            Token::Ident(_) | Token::Colon => {
                let path = self.parse_path()?;

                if allow_struct_init && self.next.value == Token::LBrace {
                    let fields = self.parse_delim(Delimiter::Brace, |parser| {
                        let mut fields = vec![];

                        while !matches!(parser.next.value, Token::RBrace | Token::Eof) {
                            let field = match parser.parse_symbol() {
                                Ok(field) => field,
                                Err(_) => {
                                    while !matches!(
                                        parser.next.value,
                                        Token::RBrace | Token::Eof | Token::Comma
                                    ) {
                                        parser.next();
                                    }

                                    if !parser.consume_if(Token::Comma) {
                                        break;
                                    }

                                    continue;
                                }
                            };

                            if parser.expect_token(Token::Colon).is_err() {
                                while !matches!(
                                    parser.next.value,
                                    Token::RBrace | Token::Eof | Token::Comma
                                ) {
                                    parser.next();
                                }

                                if !parser.consume_if(Token::Comma) {
                                    break;
                                }

                                continue;
                            }

                            let init = match parser.parse_expr(true) {
                                Ok(init) => init,
                                Err(_) => {
                                    while !matches!(
                                        parser.next.value,
                                        Token::RBrace | Token::Eof | Token::Comma
                                    ) {
                                        parser.next();
                                    }

                                    if !parser.consume_if(Token::Comma) {
                                        break;
                                    }

                                    continue;
                                }
                            };

                            fields.push(ast::StructInitField { field, init });

                            if !parser.consume_if(Token::Comma) {
                                break;
                            }
                        }

                        fields
                    })?;

                    ast::ExprKind::StructInit(ast::StructInit { path, fields })
                } else {
                    ast::ExprKind::Path(path)
                }
            }
            Token::LPar => {
                let mut trailing_comma = false;
                let mut exprs = vec![];
                self.parse_delim(Delimiter::Paren, |parser| {
                    while !matches!(parser.next.value, Token::RPar | Token::Eof) {
                        match parser.parse_expr(true) {
                            Ok(ok) => exprs.push(ok),
                            Err(_) => {
                                while !matches!(
                                    parser.next.value,
                                    Token::RPar | Token::Eof | Token::Comma
                                ) {
                                    parser.next();
                                }
                            }
                        }
                        trailing_comma = parser.consume_if(Token::Comma);
                        if !trailing_comma {
                            break;
                        }
                    }
                })?;
                if exprs.len() == 1 && !trailing_comma {
                    ast::ExprKind::Paren(Box::new(exprs.remove(0)))
                } else {
                    ast::ExprKind::Tuple(exprs)
                }
            }
            _ => {
                self.ctx.report(ExpectedExpression {
                    node: self.next.node,
                    found: self.next.value,
                });
                self.next();
                ast::ExprKind::Literal(ast::Literal::Bool(false))
            }
        };

        Ok(ast::Expr {
            node: self.ctx.join(start, self.previous.node),
            kind,
        })
    }

    fn parse_lambda(&mut self) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;
        self.expect_token(Token::Or)?;

        let mut args = vec![];
        while !matches!(self.next.value, Token::Or | Token::Eof) {
            let name = match self.parse_symbol() {
                Ok(name) => name,
                Err(_) => {
                    while !matches!(self.next.value, Token::Or | Token::Eof | Token::Comma) {
                        self.next();
                    }

                    if !self.consume_if(Token::Comma) {
                        break;
                    }

                    continue;
                }
            };

            let ty = if self.consume_if(Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };

            args.push(ast::LambdaArg { name, ty });

            if !self.consume_if(Token::Comma) {
                break;
            }
        }

        self.expect_token(Token::Or)?;

        if !self.next.value.delim_open() {
            self.ctx.report(MissingDelimiters {
                node: self.previous.node.after(),
                suggestion: "add lambda captures",
                missing: "lambda captures",
                delims: "[]",
            });
            return Err(());
        }

        let mut captures = vec![];
        self.parse_delim(Delimiter::Bracket, |parser|{
            while !matches!(parser.next.value, Token::RBracket | Token::Eof) {
                let kind = if parser.consume_if(Token::Ampersand) {
                    let ref_kind = match parser.next.value {
                        Token::Ident("raw") => {
                            parser.next();
                            ast::RefKind::Ptr
                        }
                        Token::Ident("pin") => {
                            parser.next();
                            ast::RefKind::Pinned
                        }
                        _ => ast::RefKind::Ref,
                    };
                    let mutability = if parser.consume_if(Token::Mut) {
                        ast::Mutability::Mut
                    } else {
                        ast::Mutability::Const
                    };
                    ast::LambdaCaptureKind::Borrow(mutability, ref_kind)
                } else {
                    ast::LambdaCaptureKind::Move
                };

                let name = match parser.parse_symbol() {
                    Ok(name) => name,
                    Err(_) => {
                        while !matches!(parser.next.value, Token::RBracket | Token::Eof | Token::Comma)
                        {
                            parser.next();
                        }

                        if !parser.consume_if(Token::Comma) {
                            break;
                        }

                        continue;
                    }
                };

                captures.push(ast::LambdaCapture { name, kind });

                if !parser.consume_if(Token::Comma) {
                    break;
                }
            }

        })?;

        let ret = if self.consume_if(Token::SmallRightArrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.next.value == Token::LBrace {
            ast::LambdaBody::Block(self.parse_block()??)
        } else {
            let expr = self.parse_expr(true)?;
            if ret.is_some() {
                self.ctx.report(LambdaExprBodyCannotHaveReturnType {
                    node: self.ctx.join(start, self.previous.node),
                    wrap_in_block: WrapExprInBraces {
                        open: expr.node.before(),
                        close: expr.node.after(),
                    },
                });
            }
            ast::LambdaBody::Expr(Box::new(expr))
        };

        Ok(ast::Expr {
            node: self.ctx.join(start, self.previous.node),
            kind: ast::ExprKind::Lambda(ast::Lambda {
                args,
                captures,
                ret,
                body,
            }),
        })
    }

    pub(super) fn parse_stmt(&mut self) -> PResult<ast::Stmt<'a>> {
        let start = self.next.node;

        let level = self.delimiter_stack.len();

        let kind = match self.next.value {
            Token::Let => {
                self.next();
                self.parse_let().map(ast::StmtKind::Let)
            }
            Token::Union
            | Token::Struct
            | Token::Enum
            | Token::Static
            | Token::Const
            | Token::Fn
            | Token::Mod
            | Token::Use => self.parse_item().map(ast::StmtKind::Item),

            Token::If
            | Token::While
            | Token::For
            | Token::Loop
            | Token::LBrace => self.parse_expr_bottom(true).map(ast::StmtKind::Block),

            _ => self.parse_expr(true).map(|expr| {
                if self.consume_if(Token::Semicolon) {
                    ast::StmtKind::ExprSemi(expr)
                } else {
                    ast::StmtKind::Expr(expr)
                }
            }),
        };
        let kind = match kind {
            Ok(ok) => ok,
            Err(err) => {
                // tries to recover by ignoring the remainder of the invalid item
                while !self.next.value.eof()
                    && ((!self.next.value.starts_item() && !self.next.value.delim_close())
                        || level > self.delimiter_stack.len())
                {
                    self.next();
                }
                return Err(err);
            }
        };

        Ok(ast::Stmt {
            node: self.ctx.join(start, self.previous.node),
            kind,
        })
    }

    fn parse_let(&mut self) -> PResult<ast::Let<'a>> {
        let name = self.parse_symbol()?;
        let ty = if self.consume_if(Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let initializer = if self.consume_if(Token::Assign) {
            Some(self.parse_expr(true)?)
        } else {
            None
        };

        Ok(ast::Let {
            name,
            ty,
            initializer,
        })
    }
}
