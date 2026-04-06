use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    context::Context,
    diag::{self, Diagnostics},
    node::Node,
    parser::{Parser, ast},
    source::{SourceMap, SrcIdx},
};

pub struct Compiler<'a> {
    sources: &'a SourceMap,
    programs: HashMap<SrcIdx, ast::Program<'a>>,
    ctx: Context<'a>,
}

impl<'a> Compiler<'a> {
    pub fn compile(sources: &'a SourceMap) -> Diagnostics<'a> {
        Self {
            sources,
            programs: HashMap::new(),
            ctx: Context::new(sources),
        }
        ._compile()
    }

    fn parse(&mut self, program: &Path, node: Option<Node>) -> Result<&ast::Program<'a>, ()> {
        match self.sources.load(program) {
            Ok(src) => {
                let program = Parser::new(self.ctx.clone(), src).parse();
                self.programs.insert(src.idx, program);
                self.programs.get(&src.idx).ok_or(())
            }
            Err(err) => {
                self.ctx.report(diag::other::FileError {
                    err,
                    file: program.to_path_buf(),
                    node,
                });
                Err(())
            }
        }
    }

    fn _compile(mut self) -> Diagnostics<'a> {
        let mut finished = HashSet::<PathBuf>::new();
        let mut pending = Vec::<(PathBuf, Option<Node>)>::new();
        pending.push(("main".into(), None));

        while let Some((path, node)) = pending.pop() {
            if finished.contains(&path) {
                continue;
            }
            let mut file = path.clone();
            file.set_extension("tw");
            let Ok(program) = self.parse(&file, node) else {
                continue;
            };

            for item in &program.0 {
                if let ast::ItemKind::Module(module) = &item.kind {
                    let mut path = path.to_path_buf();
                    path.set_file_name(module.name.name);

                    if finished.contains(&path) {
                        continue;
                    }
                    pending.push((path, Some(item.node)));
                }
            }

            finished.insert(path);
        }
        self.ctx.diag.take()
    }
}
