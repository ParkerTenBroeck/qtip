pub mod lex;
pub mod parse;
pub mod other;

use annotate_snippets::*;

use crate::{context::Context, node::Node, source::SrcIdx};

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
            f.write_str(&renderer.render(diag))?;
            writeln!(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

pub trait Diagnostic {
    fn to_diag<'a>(self, ctx: &Context<'a>) -> Diag<'a>;
}

pub trait Subdiagnostic {
    fn add_to_diag<'a>(
        self,
        ctx: &Context<'a>,
        group: &mut Group<'a>,
        groups: &mut Diag<'a>,
    );
}

pub fn append_to_group<'a>(group: &mut Group<'a>, element: impl Into<Element<'a>>) {
    let current = std::mem::replace(group, Group::with_level(Level::ERROR));
    *group = current.element(element);
}

pub fn annotation_snippets<'a>(
    ctx: &Context<'a>,
    items: impl IntoIterator<Item = (Node, Annotation<'a>)>,
) -> Vec<Snippet<'a, Annotation<'a>>> {
    let mut grouped: Vec<(SrcIdx, Vec<Annotation<'a>>)> = Vec::new();

    for (node, annotation) in items {
        if let Some((_, annotations)) = grouped.iter_mut().find(|(src, _)| *src == node.src) {
            annotations.push(annotation);
        } else {
            grouped.push((node.src, vec![annotation]));
        }
    }

    grouped
        .into_iter()
        .map(|(src_idx, annotations)| {
            let src = ctx.sources.get_idx(src_idx).unwrap();
            Snippet::source(&src.contents)
                .path(src.path.display().to_string())
                .annotations(annotations)
        })
        .collect()
}

pub fn patch_snippets<'a>(
    ctx: &Context<'a>,
    items: impl IntoIterator<Item = (Node, Patch<'a>)>,
) -> Vec<Snippet<'a, Patch<'a>>> {
    let mut grouped: Vec<(SrcIdx, Vec<Patch<'a>>)> = Vec::new();

    for (node, patch) in items {
        if let Some((_, patches)) = grouped.iter_mut().find(|(src, _)| *src == node.src) {
            patches.push(patch);
        } else {
            grouped.push((node.src, vec![patch]));
        }
    }

    grouped
        .into_iter()
        .map(|(src_idx, patches)| {
            let src = ctx.sources.get_idx(src_idx).unwrap();
            Snippet::source(&src.contents)
                .path(src.path.display().to_string())
                .patches(patches)
        })
        .collect()
}

