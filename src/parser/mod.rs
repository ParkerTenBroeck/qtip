pub mod ast;

use crate::{
    context::Context,
    diag::{
        Diagnostic,
        parse::{ExpectedSemi, ExpectedSymbol},
    },
    lex::{Lexer, Token},
    node::Node,
    parser::ast::BinOp,
    source::Source,
    span::Span,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct N<T> {
    value: T,
    node: Node,
}

pub struct Parser<'a> {
    src: &'a Source,
    ctx: Context<'a>,

    lexer: Lexer<'a>,
    previous: N<Token<'a>>,
    current: N<Token<'a>>,
    next: N<Token<'a>>,
}

type PResult<T> = Result<T, Box<dyn Diagnostic>>;

impl<'a> Parser<'a> {
    pub fn new(ctx: Context<'a>, src: &'a Source) -> Self {
        let mut parser = Self {
            src,
            ctx,
            lexer: Lexer::new(&src.contents),
            previous: Default::default(),
            current: Default::default(),
            next: Default::default(),
        };
        parser.next();
        parser.next();

        parser
    }

    fn next(&mut self) -> N<Token<'a>> {
        use crate::diag::lex::*;

        self.previous = self.current;
        self.current = self.next;
        self.next = loop {
            match self.lexer.next_token() {
                Ok(ok) => {
                    break N {
                        value: ok.val,
                        node: Node {
                            span: ok.span,
                            src: self.src.idx,
                            parent: None,
                        },
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
        self.previous
    }

    pub fn node(&mut self, start: Span, end: Span) -> Node {
        Node {
            span: Span {
                start: start.start,
                end: end.end,
            },
            src: self.src.idx,
            parent: None,
        }
    }

    pub fn parse(&mut self) -> ast::Program<'a> {
        let mut program = ast::Program(vec![]);

        while self.current.value != Token::Eof {
            program.0.push(self.parse_item());
        }

        program
    }

    fn parse_item(&mut self) -> ast::Item<'a> {
        let start = self.current.node;

        let vis = self.parse_vis();
        let kind = match self.next().value {
            Token::Union => todo!(),
            Token::Struct => todo!(),
            Token::Enum => todo!(),
            Token::Static => todo!(),
            Token::Const => todo!(),
            Token::Mod => ast::ItemKind::Module(self.parse_mod()),
            Token::Fn => ast::ItemKind::Fn(self.parse_fn()),
            _ => todo!(),
        };
        ast::Item {
            node: self.ctx.join(start, self.previous.node),
            kind,
            vis,
        }
    }

    fn parse_mod(&mut self) -> ast::Module<'a> {
        let name = self.parse_symbol();
        self.expect_semi();
        ast::Module { name }
    }

    fn expect_semi(&mut self) {
        if self.current.value == Token::Semicolon {
            self.next();
            return;
        }
        self.ctx.report(ExpectedSemi {
            node: self.previous.node.after().after(),
            found: self.current.value,
            found_node: self.current.node
        });
    }

    fn parse_fn_params(&mut self) -> Vec<ast::FnParam<'a>> {
        if !matches!(self.current.value, Token::LPar) {
            return vec![];
        }
        self.next();

        let mut params = vec![];

        // while self.next().value != Token::RPar {
        //     // TODO report error
        // }
        self.next();

        params
    }

    fn parse_fn(&mut self) -> ast::Fn<'a> {
        let name = self.parse_symbol();

        let params = self.parse_fn_params();

        let ret = if matches!(self.current.value, Token::SmallRightArrow) {
            self.next();
            Some(self.parse_type())
        } else {
            None
        };

        let body = if matches!(self.current.value, Token::Semicolon) {
            None
        } else {
            Some(self.parse_expr())
        };

        ast::Fn {
            name,
            params,
            ret,
            body,
        }
    }

    fn parse_block(&mut self) -> ast::Block<'a> {
        self.next();

        let mut stmts = vec![];

        while self.current.value != Token::RBrace {
            stmts.push(self.parse_stmt())
        }

        self.next();

        ast::Block { stmts }
    }

    fn parse_expr(&mut self) -> ast::Expr<'a> {
        self.parse_expr_binop(0)
    }

    fn parse_expr_binop(&mut self, min_prec: u32) -> ast::Expr<'a> {
        let start = self.current.node;

        let mut lhs = self.parse_expr_2();

        loop {
            let op = match self.current.value {
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
                Token::MinusAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                    BinOp::PlusAssign
                }
                Token::TimesAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                    BinOp::PlusAssign
                }
                Token::DivideAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                    BinOp::PlusAssign
                }
                Token::ModuloAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                    BinOp::PlusAssign
                }
                Token::OrAssign if BinOp::PlusAssign.precedence() >= min_prec => BinOp::PlusAssign,
                Token::AndAssign if BinOp::PlusAssign.precedence() >= min_prec => BinOp::PlusAssign,
                Token::XorAssign if BinOp::PlusAssign.precedence() >= min_prec => BinOp::PlusAssign,
                // Token::ShiftRightAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                //     BinOp::PlusAssign
                // }
                // Token::ShiftLeftAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                //     BinOp::PlusAssign
                // }
                _ => break,
            };
            self.next();

            let rhs = self.parse_expr_binop(op.precedence() + op.right_to_left() as u32);
            lhs = ast::Expr {
                node: self.ctx.join(start, self.previous.node),
                kind: ast::ExprKind::BinOp {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                },
            }
        }

        lhs
    }

    fn parse_expr_2(&mut self) -> ast::Expr<'a> {
        //todo parse as
        self.parse_expr_3()
    }

    fn parse_expr_3(&mut self) -> ast::Expr<'a> {
        // parse regular unop
        self.parse_expr_bottom()
    }

    fn parse_expr_labled(&mut self) -> ast::Expr<'a> {
        todo!()
    }

    fn parse_expr_bottom(&mut self) -> ast::Expr<'a> {
        let start = self.current.node;
        let kind = match self.current.value {
            Token::LBrace => ast::ExprKind::Block(self.parse_block()),
            Token::Label(_) => {
                // self.parse_expr_labled();
                todo!()
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
            Token::Ident(_) => ast::ExprKind::Path(self.parse_symbol()),
            Token::LPar => {
                self.next();

                let paren = ast::ExprKind::Paren(Box::new(self.parse_expr()));
                // TODO actually check if this is RPar
                self.next();
                paren
            }
            _ => todo!(),
        };

        ast::Expr {
            node: self.ctx.join(start, self.previous.node),
            kind,
        }
    }

    fn parse_stmt(&mut self) -> ast::Stmt<'a> {
        let start = self.current.node;

        let kind = match self.current.value {
            Token::Let => ast::StmtKind::Let(self.parse_let()),
            Token::Union
            | Token::Struct
            | Token::Enum
            | Token::Static
            | Token::Const
            | Token::Fn => ast::StmtKind::Item(self.parse_item()),
            _ => {
                let expr = self.parse_expr();
                if self.current.value == Token::Semicolon {
                    self.next();
                }
                ast::StmtKind::Expr(expr)
            }
        };

        ast::Stmt {
            node: self.ctx.join(start, self.previous.node),
            kind,
        }
    }

    fn parse_let(&mut self) -> ast::Let<'a> {
        todo!()
    }

    fn parse_type(&mut self) -> ast::Type<'a> {
        ast::Type {
            name: self.parse_symbol(),
        }
    }

    fn parse_symbol(&mut self) -> ast::Symbol<'a> {
        if let Token::Ident(name) = self.current.value {
            self.next();
            ast::Symbol {
                name,
                node: self.current.node,
            }
        } else {
            self.ctx.report(ExpectedSymbol {
                node: self.current.node,
                found: self.current.value,
            });
            ast::Symbol {
                name: "<<ERROR>>",
                node: self.current.node,
            }
        }
    }

    fn parse_vis(&mut self) -> ast::Vis {
        if self.current.value == Token::Ident("pub") {
            self.next();
            ast::Vis::Pub
        } else {
            ast::Vis::Priv
        }
    }
}
