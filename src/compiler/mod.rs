use std::path::Path;

use crate::{
    context::Context,
    diag::{self, Diagnostics},
    node::Node,
    parser::{Parser, ast},
    source::SourceMap,
};

pub struct Compiler<'a> {
    sources: &'a SourceMap,
    programs: Vec<ast::Program<'a>>,
    ctx: Context<'a>,
}

impl<'a> Compiler<'a> {
    pub fn compile(sources: &'a SourceMap) -> Diagnostics<'a> {
        Self {
            sources,
            programs: vec![],
            ctx: Context::new(sources),
        }
        ._compile()
    }

    fn parse(&mut self, program: &Path, node: Option<Node>) {
        match self.sources.load(program) {
            Ok(src) => {
                let program = Parser::new(self.ctx.clone(), src).parse();
                self.programs.push(program);
            }
            Err(err) => {
                self.ctx.report(diag::FileError {
                    err,
                    file: program.to_path_buf(),
                    node,
                });
            }
        }
    }

    fn _compile(mut self) -> Diagnostics<'a> {
        self.parse("main.tw".as_ref(), None);
        self.ctx.diag.take()
    }
}
