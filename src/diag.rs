use std::path::PathBuf;

use annotate_snippets::*;
use proc_macros::Diagnostic;

use crate::{context::Context, node::Node};

pub type Diag<'a> = Vec<Group<'a>>;

#[derive(Default)]
pub struct Diagnostics<'a> {
    diags: Vec<Diag<'a>>,
}

impl<'a> Diagnostics<'a> {
    pub fn new() -> Self {
        Diagnostics { diags: vec![] }
    }

    pub fn report(&mut self, ctx: &Context<'a>, report: impl Diagnostic) {
        self.diags.push(report.to_diag(ctx));
    }
}

impl<'a> std::fmt::Display for Diagnostics<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let renderer = annotate_snippets::Renderer::styled();
        for diag in &self.diags {
            f.write_str(&renderer.render(&diag))?;
            writeln!(f)?;
        }
        Ok(())
    }
}

pub trait Diagnostic {
    fn to_diag<'a>(self, ctx: &Context<'a>) -> Diag<'a>;
}

#[derive(Diagnostic)]
#[diag("{msg}")]
#[note("meow")]
pub struct LexerError {
    #[primary_node]
    #[label("here")]
    #[note("weee")]
    pub node: Node,
    pub msg: String,
}


impl Diagnostic for LexerError {
    #[allow(unused)]
    fn to_diag<'a>(self, ctx: &Context<'a>) -> Diag<'a> {
        let Self { node, msg } = self;
        let mut group = Group::with_title(Level::ERROR.primary_title(format!("{msg}")));
        group = group.element(Level::NOTE.message(format!("meow")));
        let src = ctx.sources.get_idx(node.src).unwrap();
        let snippet = Snippet::source(&src.contents).path(src.path.as_os_str().to_str().unwrap());
        group = group.element(
            Level::NOTE
                .primary_title(format!("weee")).element(Level::NOTE.message("meow"))
                .annotation(AnnotationKind::Primary.span(node.range.into())),
        );
        vec![group]
    }
}

// impl Diagnostic for LexerError {
//     fn to_diag<'a>(self, ctx: &Context<'a>) -> Diag<'a> {
//         let group = Group::with_title(Level::ERROR.primary_title(format!("failed to load file")));

//         let src = ctx.sources.get_idx(self.node.src).unwrap();
//         let snippet = Snippet::source(&src.contents)
//             .path(src.path.as_os_str().to_str().unwrap())
//             .annotation(AnnotationKind::Primary.span(self.node.range.into()));

//         let group = group.element(snippet).element(Level::HELP.message("me"));

//         let snippet = Snippet::source(&src.contents)
//             .path(src.path.as_os_str().to_str().unwrap())
//             .annotation(
//                 AnnotationKind::Context
//                     .span(self.node.range.into())
//                     .label("meow"),
//             );

//         let group = group.element(snippet);
//         // .annotation(AnnotationKind::Context.span(span).)

//         vec![
//             group,
//             Group::with_title(Level::HELP.secondary_title("emwo")),
//         ]
//     }
// }

pub struct FileError {
    pub err: std::io::Error,
    pub file: PathBuf,
    pub node: Option<Node>,
}

impl Diagnostic for FileError {
    fn to_diag<'a>(self, ctx: &Context<'a>) -> Diag<'a> {
        let snippet = self.node.map(|node| {
            let src = ctx.sources.get_idx(node.src).unwrap();
            Snippet::source(&src.contents)
                .path(src.path.as_os_str().to_str().unwrap())
                .annotation(AnnotationKind::Primary.span(node.range.into()))
            // .annotation(AnnotationKind::Context.span(span).)
        });

        // 'level{

        // }
        
        // self.node.unwrap().src
        // Level::NOTE.primary_title("text").element(section)

        Group::with_title(Level::HELP.secondary_title("emwo"));
        // Level::ERROR
        //     .primary_title(format!(
        //         "failed to load file {}: {}",
        //         self.file.display(),
        //         self.err
        //     )).element(Level::HELP)
        //     .elements(snippet)
        todo!()
    }
}

pub struct ExpectedItem {}
