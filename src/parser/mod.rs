pub mod ast;
pub mod expr;
pub mod item;
pub mod ty;

use crate::{
    context::Context,
    diag::parse::*,
    lex::{Lexer, Token},
    node::Node,
    source::Source,
    span::Span,
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

    fn parse_delim<R>(
        &mut self,
        delim: Delimiter,
        func: impl FnOnce(&mut Self) -> R,
    ) -> PResult<R> {
        let (open, close) = match delim {
            Delimiter::Paren => (Token::LPar, Token::RPar),
            Delimiter::Brace => (Token::LBrace, Token::RBrace),
            Delimiter::Bracket => (Token::LBracket, Token::RBracket),
        };

        let level = self.delimiter_stack.len();

        let open_got = self.next;

        if self.next.value.delim_open() {
            self.next();
        } else {
            self.ctx.report(UnexpectedToken {
                node: self.next.node,
                found: self.next.value,
                expected: open,
            });
            return Err(());
        }

        let ret = func(self);

        if !self.next.value.delim_close() && !self.next.value.eof() {
            self.ctx.report(UnexpectedToken {
                node: self.next.node,
                found: self.next.value,
                expected: close,
            });
        }
        while !self.next.value.eof() && self.delimiter_stack.len() > level {
            self.next();
        }

        if self.previous.value.delim_close() && self.delimiter_stack.len() == level {
            
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
            if !self.next.value.delim() {
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

    fn parse_fn_param(&mut self) -> PResult<ast::FnParam<'a>> {
        let start = self.next.node;
        let name = self.parse_symbol()?;
        self.expect_token(Token::Colon)?;
        let ty = self.parse_type()?;
        let node = self.ctx.join(start, self.previous.node);

        Ok(ast::FnParam { node, name, ty })
    }

    fn parse_block(&mut self) -> PResult<PResult<ast::Block<'a>>> {
        let mut stmts = vec![];

        if !self.next.value.delim_open() {
            self.ctx.report(MissingDelimiters {
                node: self.previous.node.after(),
                suggestion: "add block",
                missing: "block",
                delims: "{}",
            });
            return Err(());
        }

        self.parse_delim(Delimiter::Brace, |parser| {
            while !matches!(parser.next.value, Token::RBrace | Token::Eof) {
                if let Ok(stmt) = parser.parse_stmt() {
                    stmts.push(stmt)
                }
            }
            Ok(ast::Block { stmts })
        })
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
            if !self.next.value.delim() {
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
