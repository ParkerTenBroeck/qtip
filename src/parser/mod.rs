pub mod ast;
pub mod diag;

use crate::{
    lex::{Lexer, Token},
    node::Node,
    parser::{ast::BinOp, diag::Diagnostics},
    span::{Span, Spanned as S},
};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    previous: S<Token<'a>>,
    current: S<Token<'a>>,

    diag: Diagnostics<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut parser = Self {
            lexer,
            previous: S::new(Token::default(), Default::default()),
            current: S::new(Token::default(), Default::default()),

            diag: Diagnostics::new(),
        };
        parser.next();

        parser
    }

    fn next(&mut self) -> S<Token<'a>> {
        self.previous = self.current;
        self.current = loop {
            match self.lexer.next_token() {
                Ok(ok) => break ok,
                Err(err) => todo!("{err:?}"),
            }
        };
        self.previous
    }

    pub fn node(&mut self, start: Span, end: Span) -> Node {
        Node {
            range: Span {
                start: start.start,
                end: end.end,
            },
            src: todo!(),
            parent: todo!(),
        }
    }

    pub fn parse(&mut self) -> ast::Program<'a> {
        let mut program = ast::Program(vec![]);

        while self.current.val != Token::Eof {
            program.0.push(self.parse_item());
        }

        program
    }

    fn parse_item(&mut self) -> ast::Item<'a> {
        let start = self.current.span;

        let vis = self.parse_vis();
        let kind = match self.next().val {
            Token::Union => todo!(),
            Token::Struct => todo!(),
            Token::Enum => todo!(),
            Token::Static => todo!(),
            Token::Const => todo!(),
            Token::Fn => ast::ItemKind::Fn(self.parse_fn()),
            _ => todo!(),
        };
        ast::Item {
            node: self.node(start, self.previous.span),
            kind,
            vis,
        }
    }

    fn parse_fn_params(&mut self) -> Vec<ast::FnParam<'a>> {
        if !matches!(self.current.val, Token::LPar) {
            return vec![];
        }
        self.next();

        let mut params = vec![];

        // while self.next().val != Token::RPar {
        //     // TODO report error
        // }
        self.next();

        params
    }

    fn parse_fn(&mut self) -> ast::Fn<'a> {
        let name = self.parse_symbol();

        let params = self.parse_fn_params();

        let ret = if matches!(self.current.val, Token::SmallRightArrow) {
            self.next();
            Some(self.parse_type())
        } else {
            None
        };

        let body = if matches!(self.current.val, Token::Semicolon) {
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

        while self.current.val != Token::RBrace {
            stmts.push(self.parse_stmt())
        }

        self.next();

        ast::Block { stmts }
    }

    fn parse_expr(&mut self) -> ast::Expr<'a> {
        self.parse_expr_binop(0)
    }

    fn parse_expr_binop(&mut self, min_prec: u32) -> ast::Expr<'a> {
        let start = self.current.span;

        let mut lhs = self.parse_expr_2();

        loop {
            let op = match self.current.val {
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
                Token::GreaterThan if BinOp::Gt.precedence() >= min_prec => BinOp::Gt,
                Token::GreaterThanEq if BinOp::Gte.precedence() >= min_prec => BinOp::Gte,
                Token::LessThan if BinOp::Lt.precedence() >= min_prec => BinOp::Lt,
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
                Token::ShiftRightAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                    BinOp::PlusAssign
                }
                Token::ShiftLeftAssign if BinOp::PlusAssign.precedence() >= min_prec => {
                    BinOp::PlusAssign
                }

                _ => break,
            };
            self.next();

            let rhs = self.parse_expr_binop(op.precedence() + op.right_to_left() as u32);
            lhs = ast::Expr {
                node: self.node(start, self.previous.span),
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
        let start = self.current.span;
        let kind = match self.current.val {
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
            node: self.node(start, self.previous.span),
            kind,
        }
    }

    fn parse_stmt(&mut self) -> ast::Stmt<'a> {
        let start = self.current.span;

        let kind = match self.current.val {
            Token::Let => ast::StmtKind::Let(self.parse_let()),
            Token::Union
            | Token::Struct
            | Token::Enum
            | Token::Static
            | Token::Const
            | Token::Fn => ast::StmtKind::Item(self.parse_item()),
            _ => {
                let expr = self.parse_expr();
                if self.current.val == Token::Semicolon {
                    self.next();
                }
                ast::StmtKind::Expr(expr)
            }
        };

        ast::Stmt {
            node: self.node(start, self.previous.span),
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
        let span = self.current.span;
        if let Token::Ident(name) = self.current.val {
            self.next();
            ast::Symbol {
                name,
                node: self.node(span, span),
            }
        } else {
            ast::Symbol {
                name: "<<ERROR>>",
                node: self.node(span, span),
            }
        }
    }

    fn parse_vis(&mut self) -> ast::Vis {
        if self.current.val == Token::Ident("pub") {
            self.next();
            ast::Vis::Pub
        } else {
            ast::Vis::Priv
        }
    }
}
