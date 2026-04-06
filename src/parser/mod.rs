pub mod ast;

use crate::{
    context::Context, diag::parse::*, lex::{Lexer, Token}, node::Node, parser::ast::Symbol, source::Source, span::Span
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct N<T> {
    value: T,
    node: Node,
}

pub struct Parser<'a> {
    src: &'a Source,
    ctx: Context<'a>,

    lexer: Lexer<'a>,
    previous: N<Token<'a>>,
    next: N<Token<'a>>,
    // next: N<Token<'a>>,

    delimiter_stack: Vec<(Delimiter, Node)>,
}

type PResult<T> = Result<T, ()>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Delimiter {
    Paren,
    Brace,
    Bracket,
}

impl<'a> Parser<'a> {
    pub fn new(ctx: Context<'a>, src: &'a Source) -> Self {
        let empty_node = N {
            value: Default::default(),
            node: Node {
                span: Span::default(),
                src: src.idx,
                parent: None,
            },
        };
        let mut parser = Self {
            src,
            ctx,
            lexer: Lexer::new(&src.contents),
            previous: empty_node,
            next: empty_node,
            delimiter_stack: vec![],
        };
        parser.next();

        parser
    }

    fn next(&mut self) -> N<Token<'a>> {
        use crate::diag::lex::*;

        self.previous = self.next;
        self.next = loop {
            match self.lexer.next_token() {
                Ok(ok) => {
                    let node = Node {
                        span: ok.span,
                        src: self.src.idx,
                        parent: None,
                    };
                    match ok.val {
                        Token::MultiLineComment(_) | Token::SingleLineComment(_) => {
                            self.ctx.report(CommentWarning { node });
                            continue;
                        }
                        _ => {}
                    }
                    break N {
                        value: ok.val,
                        node,
                    };
                }
                Err(err) => self.ctx.report(LexerError {
                    msg: err.val,
                    node: Node {
                        span: err.span,
                        src: self.src.idx,
                        parent: None,
                    },
                }),
            }
        };

        match self.previous.value {
            Token::LPar | Token::LBrace | Token::LBracket => {
                let delim = match self.previous.value {
                    Token::LPar => Delimiter::Paren,
                    Token::LBrace => Delimiter::Brace,
                    Token::LBracket => Delimiter::Bracket,
                    _ => unreachable!(),
                };
                self.delimiter_stack.push((delim, self.previous.node));
            }
            Token::RPar | Token::RBrace | Token::RBracket => {
                let delim = match self.previous.value {
                    Token::RPar => Delimiter::Paren,
                    Token::RBrace => Delimiter::Brace,
                    Token::RBracket => Delimiter::Bracket,
                    _ => unreachable!(),
                };
                match self.delimiter_stack.pop() {
                    None => {
                        self.ctx.report(UnexpectedClosingDelim {
                            delim: self.previous.value,
                            node: self.previous.node,
                        });
                    }
                    Some((expected, other)) if expected != delim => {
                        self.ctx.report(MismatchedDelims {
                            lhs: other,
                            rhs: self.previous.node,
                        });
                    }
                    _ => {}
                }
            }
            Token::Eof => {
                if !self.delimiter_stack.is_empty() {
                    self.ctx.report(UnclosedDelimiters {
                        node: self.previous.node,
                        unclosed: self.delimiter_stack.drain(..).map(|(_, n)| n).collect(),
                    });
                }
            }
            _ => {}
        }
        self.previous
    }

    pub fn parse(&mut self) -> ast::Program<'a> {
        ast::Program(self.parse_item_list())
    }

    pub fn parse_item_list(&mut self) -> Vec<ast::Item<'a>> {
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

        list
    }

    fn parse_item(&mut self) -> PResult<ast::Item<'a>> {
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

    fn expect_semi(&mut self) {
        if self.next.value == Token::Semicolon {
            self.next();
            return;
        }
        self.ctx.report(ExpectedSemi {
            node: self.previous.node.after(),
            found: self.next.value,
            found_node: self.next.node,
        });
    }

    fn parse_delim<R>(&mut self, delim: Delimiter, func: impl FnOnce(&mut Self) -> R) -> PResult<R> {
        let (open, close) = match delim {
            Delimiter::Paren => (Token::LPar, Token::RPar),
            Delimiter::Brace => (Token::LBrace, Token::RBrace),
            Delimiter::Bracket => (Token::LBracket, Token::RBracket),
        };

        let level = self.delimiter_stack.len();
        
        if self.next.value.delim_open(){
            self.next();
        } else {
            self.ctx.report(UnexpectedToken{
                node: self.next.node,
                found: self.next.value,
                expected: open,
            });
            return Err(())
        }
        
        let ret = func(self);
        
        if !self.next.value.delim_close(){
            self.ctx.report(UnexpectedToken{
                node: self.next.node,
                found: self.next.value,
                expected: close,
            });
        }
        while !self.next.value.eof() && self.delimiter_stack.len() > level {
            dbg!(self.next());
        }
        
        Ok(ret)
    }

    fn expect_token(&mut self, expected: Token<'a>) -> PResult<()> {
        if self.next.value != expected {
            self.ctx.report(UnexpectedToken {
                node: self.next.node,
                found: self.next.value,
                expected,
            });
            if !self.next.value.delim(){
                self.next();
            }
            Err(())
        } else {
            self.next();
            Ok(())
        }
    }

    fn consume_if(&mut self, token: Token<'a>) -> bool {
        if self.next.value == token {
            self.next();
            true
        } else {
            false
        }
    }

    fn parse_fn_params(&mut self) -> PResult<Vec<ast::FnParam<'a>>> {
        let mut params = vec![];

        self.parse_delim(Delimiter::Paren, |parser| {
            while !matches!(parser.next.value, Token::RPar | Token::Eof) {
                let start = parser.next.node;
                let name = parser.parse_symbol()?;
                parser.expect_token(Token::Colon)?;
                let ty = parser.parse_type()?;
                let node = parser.ctx.join(start, parser.previous.node);

                params.push(ast::FnParam { node, name, ty });

                if !parser.consume_if(Token::Comma) {
                    break;
                }
            }
            Ok(())
        })??;

        Ok(params)
    }

    fn parse_fn(&mut self) -> PResult<ast::Fn<'a>> {
        let name = self.parse_symbol()?;
        let params = self.parse_fn_params()?;
        let ret = if self.consume_if(Token::SmallRightArrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = if self.consume_if(Token::Semicolon) {
            None
        } else {
            Some(self.parse_block()?)
        };

        Ok(ast::Fn {
            name,
            params,
            ret,
            body,
        })
    }

    fn parse_block(&mut self) -> PResult<ast::Block<'a>> {
        let mut stmts = vec![];

        self.parse_delim(Delimiter::Brace, |parser| {
            while !matches!(parser.next.value, Token::RBrace | Token::Eof) {
                stmts.push(parser.parse_stmt()?)
            }
            Ok(())
        })??;

        Ok(ast::Block { stmts })
    }

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
            ast::ExprKind::If(Box::new(cond), block, chain, label)
        } else {
            ast::ExprKind::Block(self.parse_block()?, None)
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
                ast::ExprKind::While(Box::new(self.parse_expr()?), self.parse_block()?, label)
            }
            Token::LBrace => ast::ExprKind::Block(self.parse_block()?, label),
            Token::Loop => ast::ExprKind::Loop(self.parse_block()?, label),
            Token::For => ast::ExprKind::For(self.parse_block()?, label),
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

    fn parse_expr_bottom(&mut self) -> PResult<ast::Expr<'a>> {
        let start = self.next.node;
        let kind = match self.next.value {
            Token::LBrace | Token::If | Token::While | Token::Loop | Token::For => {
                return self.parse_expr_labled(None);
            }
            Token::At => {
                self.next();
                let sym = self.parse_symbol()?;
                return self.parse_expr_labled(Some(ast::Label { sym }));
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

    fn parse_stmt(&mut self) -> PResult<ast::Stmt<'a>> {
        let start = self.next.node;

        let level = self.delimiter_stack.len();

        let kind = match self.next.value {
            Token::Let => self.parse_let().map(ast::StmtKind::Let),
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
                    && (!self.next.value.starts_item() || level > self.delimiter_stack.len())
                {
                    self.next();
                }
                return Err(err)
            }
        };

        Ok(ast::Stmt {
            node: self.ctx.join(start, self.previous.node),
            kind,
        })
    }

    fn parse_let(&mut self) -> PResult<ast::Let<'a>> {
        let name = self.parse_symbol()?;
        self.expect_token(Token::Colon)?;
        let ty = self.parse_type()?;
        let initializer = if self.consume_if(Token::Equals) {
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

    fn parse_type(&mut self) -> PResult<ast::Type<'a>> {
        match self.next.value{
            Token::Ident(name) => {
                self.next();
                Ok(ast::Type{
                    name: Symbol{
                        name,
                        node: self.previous.node
                    }
                })
            }
            _ => {
                self.ctx.report(ExpectedType {
                    node: self.next.node,
                    found: self.next.value,
                });
                if !self.next.value.delim(){
                    self.next();
                }
                Err(())
            }
        }
    }

    fn parse_symbol(&mut self) -> PResult<ast::Symbol<'a>> {
        if let Token::Ident(name) = self.next.value {
            self.next();
            Ok(ast::Symbol {
                name,
                node: self.previous.node,
            })
        } else {
            self.ctx.report(ExpectedSymbol {
                node: self.next.node,
                found: self.next.value,
            });
            if !self.next.value.delim(){
                self.next();
            }
            Err(())
        }
    }

    fn parse_vis(&mut self) -> ast::Vis {
        if self.next.value == Token::Ident("pub") {
            self.next();
            ast::Vis::Pub
        } else {
            ast::Vis::Priv
        }
    }
}
