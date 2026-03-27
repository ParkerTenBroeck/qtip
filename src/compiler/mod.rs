use std::path::Path;

use crate::{parser::{ast, diag::Diagnostics}, source::SourceMap};

pub struct Compiler<'a> {
    sources: &'a SourceMap,
    programs: Vec<ast::Program<'a>>,
    diag: Diagnostics<'a>,
}

impl<'a> Compiler<'a>{
    pub fn compile(sources: &'a SourceMap) -> Diagnostics<'a>{
        Self{
            sources,
            programs: vec![],
            diag: Diagnostics::new(),
        }._compile()
    }

    fn parse(&mut self, program: &Path) {
        // match self.sources.load(program){
        //     Ok(src) => src.contents,
        //     Err(_) => todo!(),
        // }
    }

    fn _compile(mut self) -> Diagnostics<'a>{
        
        self.diag
    }
}
