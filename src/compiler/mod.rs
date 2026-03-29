use std::path::Path;

use crate::{context::Context, diag::Diagnostics, parser::{Parser, ast}, source::SourceMap};

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

    fn parse(&mut self, program: &Path) {
        match self.sources.load(program){
            Ok(src) => {
                let program = Parser::new(self.ctx.clone(), src).parse();
                self.programs.push(program);
            },
            Err(_) => todo!(),
        }
    }
    
    fn _compile(mut self) -> Diagnostics<'a> {
        self.parse("main.tw".as_ref());
        self.ctx.diag.take()
    }
}
