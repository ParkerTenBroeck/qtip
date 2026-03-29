use annotate_snippets::*;

use crate::{node::Node, source::SourceMap};

#[derive(Default)]
pub struct Diagnostics<'a> {
    diags: Vec<Group<'a>>,
}

impl<'a> Diagnostics<'a> {
    pub fn new() -> Self {
        Diagnostics { diags: vec![] }
    }

    pub fn report(&mut self, sources: &'a SourceMap, report: impl Diagnostic<'a>) {
        self.diags.push(report.to_diag(sources));
    }
}

impl<'a> std::fmt::Display for Diagnostics<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&annotate_snippets::Renderer::styled().render(&self.diags))
    }
}

pub trait Diagnostic<'a> {
    fn to_diag(self, sources: &'a SourceMap) -> Group<'a>;
}

pub struct LexerError {
    pub msg: String,
    pub node: Node,
}

impl<'a> Diagnostic<'a> for LexerError {
    fn to_diag(self, sources: &'a SourceMap) -> Group<'a> {
        let src = sources.get_idx(self.node.src).unwrap();
        let snippet = Snippet::source(&src.contents)
            .path(src.path.as_os_str().to_str().unwrap())
            .annotation(
                AnnotationKind::Primary
                    .span(self.node.range.into())
            );

        Level::ERROR.primary_title(self.msg).element(snippet)
    }
}
