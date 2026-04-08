use crate::parser::*;

use crate::{diag::parse::*, lex::Token};

impl<'a> Parser<'a> {
    fn parse_expr(&mut self) -> PResult<ast::Expr<'a>> {
        self.parse_expr_binop(0)
    }

    fn parse_expr_binop(&mut self, min_prec: u32) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;

        let mut lhs = self.parse_expr_2()?;

        use ast::BinOp;
        loop {
            let op = match self.next.value {
                Token::Plus if BinOp::Add.precedence() >= min_prec => BinOp::Add,
                Token::Minus if BinOp::Sub.precedence() >= min_prec => BinOp::Sub,
                Token::Star if BinOp::Mul.precedence() >= min_prec => BinOp::Mul,
                Token::Slash if BinOp::Div.precedence() >= min_prec => BinOp::Div,
                Token::Percent if BinOp::Rem.precedence() >= min_prec => BinOp::Rem,
                Token::LogicalOr if BinOp::Or.precedence() >= min_prec => BinOp::Or,
                Token::BitwiseOr if BinOp::Or.precedence() >= min_prec => BinOp::Or,
                Token::Ampersand if BinOp::And.precedence() >= min_prec => BinOp::And,
                Token::LogicalAnd if BinOp::And.precedence() >= min_prec => BinOp::And,
                Token::BitwiseXor if BinOp::Xor.precedence() >= min_prec => BinOp::Xor,
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

            let rhs = self.parse_expr_binop(op.precedence() + op.right_to_left() as u32)?;
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

    fn parse_expr_2(&mut self) -> PResult<ast::Expr<'a>> {
        //todo parse as
        self.parse_expr_3()
    }

    fn parse_expr_3(&mut self) -> PResult<ast::Expr<'a>> {
        // parse regular unop
        self.parse_expr_bottom()
    }

    fn parse_if_chain(&mut self, label: Option<ast::Label<'a>>) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;

        let kind = if self.consume_if(Token::If) {
            let cond = self.parse_expr()?;
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
                ast::ExprKind::While(Box::new(self.parse_expr()?), self.parse_block()??, label)
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
    
    fn parse_label_opt(&mut self) -> PResult<Option<ast::Label<'a>>>{
        if self.consume_if(Token::At){
            Ok(Some(ast::Label{
                sym: self.parse_symbol()?,
            }))
        }else{
            Ok(None)
        }
    } 

    fn parse_expr_bottom(&mut self) -> PResult<ast::Expr<'a>> {

        let start = self.next.node;
        let kind = match self.next.value {
            Token::LBrace | Token::If | Token::While | Token::Loop | Token::For => {
                return self.parse_expr_labled(None);   
            }
            Token::Return => {
                self.next();
                ast::ExprKind::Return(Box::new(self.parse_expr()?))
            }
            Token::Continue => {
                self.next();
                let label = self.parse_label_opt()?;
                ast::ExprKind::Continue(Box::new(self.parse_expr()?), label)
            }
            Token::Break => {
                self.next();
                let label = self.parse_label_opt()?;
                ast::ExprKind::Break(Box::new(self.parse_expr()?), label)
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
            Token::Ident(_) => ast::ExprKind::Path(self.parse_symbol()?),
            Token::LPar => self.parse_delim(Delimiter::Paren, |parser| {
                Ok(ast::ExprKind::Paren(Box::new(parser.parse_expr()?)))
            })??,
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
            | Token::Mod => self.parse_item().map(ast::StmtKind::Item),
            _ => self.parse_expr().map(|expr| {
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
            Some(self.parse_expr()?)
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
