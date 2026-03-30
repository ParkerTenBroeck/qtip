use std::{path::PathBuf, process::Output};

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

    pub fn report(&mut self, sources: &'a SourceMap, report: impl Diagnostic) {
        self.diags.push(report.to_diag(sources));
    }
}

impl<'a> std::fmt::Display for Diagnostics<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let renderer = annotate_snippets::Renderer::styled();
        f.write_str(&renderer.render(&self.diags))
    }
}

pub trait Diagnostic {
    fn to_diag<'a>(self, sources: &'a SourceMap) -> Group<'a>;
}

macro_rules! diagnostic {
    (
        $(#$top_attr:tt)*
        $vis:vis struct $name:ident {
            $(
                $(#$field_attr:tt)*
                pub $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        $vis struct $name {
            $(
                pub $field : $ty
            ),*
        }

        impl Diagnostic for $name {
            #[allow(unused)]
            fn to_diag<'a>(self, sources: &'a SourceMap) -> Group<'a> {
                let Self {
                    $(
                        $field
                    ),*
                } = self;

                let diag = Level::ERROR;

                diagnostic!(diag, @top_first: $(#$top_attr)*);
                $(
                    diagnostic!(diag, sources, @field:  $(#$field_attr)*, $field : $ty);
                )*
                diagnostic!(diag, @top_last: $(#$top_attr)*);

                diag
            }
        }
    };


    // -------------------------

    ($diag:ident, @top_first: #[diag($lit:tt)] $(#$rem:tt)*) => {
        let $diag = $diag.primary_title(format!($lit));
        diagnostic!($diag, @top_first: $(#$rem)*);
    };

    ($diag:ident, @top_first: #[note($lit:tt)] $(#$rem:tt)*) => {
        diagnostic!($diag, @top_first: $(#$rem)*);
    };
    
    ($diag:ident, @top_first: ) => {};

    ($diag:ident, @top_first: $(#$top_attr:tt)*) => {
        compile_error!(stringify!(unknown diagnostic attribute $(#$top_attr)*))
    };

    // -------------------------

    ($diag:ident, @top_last: #[diag($lit:tt)] $(#$rem:tt)*) => {
        diagnostic!($diag, @top_last: $(#$rem)*);
    };

    ($diag:ident, @top_last: #[note($lit:tt)] $(#$rem:tt)*) => {
        let $diag = $diag.element(Level::HELP.message(format!($lit)));
        diagnostic!($diag, @top_last: $(#$rem)*);
    };
    
    ($diag:ident, @top_last: ) => {};

    ($diag:ident, @top_last: $(#$top_attr:tt)*) => {
        compile_error!(stringify!(unknown diagnostic attribute $(#$top_attr)*))
    };
    

    ($diag:ident, $sources:ident, @field: $(#$rem:tt)*, $field:ident : Node) => {
        let snippet = $field.map(|node| {
            let src = $sources.get_idx(node.src).unwrap();
            Snippet::source(&src.contents)
                .path(src.path.as_os_str().to_str().unwrap())
                .annotation(AnnotationKind::Primary.span(node.range.into()))
        });
        diagnostic!($diag, $sources, @field: $(#$rem)*, $field : $ty);
    };
    
    ($diag:ident, $sources:ident, @field: #[primary_node] $(#$rem:tt)*, $field:ident : $ty:ty) => {
        
        let src = $sources.get_idx($field.src).unwrap();
        let snippet = Snippet::source(&src.contents)
            .path(src.path.as_os_str().to_str().unwrap())
            .annotation(AnnotationKind::Primary.span($field.range.into()));

        let $diag = $diag.element(snippet);
        
        diagnostic!($diag, $sources, @field: $(#$rem)*, $field : $ty);
    };
    
    ($diag:ident, $sources:ident, @field: , $field:ident : $ty:ty) => {

    };
    
    ($diag:ident, $sources:ident, @field: $(#$field_attr:tt)*, $field:ident : $ty:ty) => {

        compile_error!(stringify!(unknown diagnostic field attribute $(#$field_attr)*, $field : $ty))
    };
}

diagnostic!(
    #[diag("{msg}")]
    #[note("meow")]
    pub struct LexerError {
        #[primary_node]
        // #[label("here")]
        pub node: Node,
        pub msg: String,
    }
);

// pub struct LexerError {
//     pub msg: String,
//     pub node: Node,
// }

// impl Diagnostic for LexerError {
//     fn to_diag<'a>(self, sources: &'a SourceMap) -> Group<'a> {
//         let src = sources.get_idx(self.node.src).unwrap();
//         let snippet = Snippet::source(&src.contents)
//             .path(src.path.as_os_str().to_str().unwrap())
//             .annotation(AnnotationKind::Primary.span(self.node.range.into()));

//         Level::ERROR.primary_title(self.msg).element(snippet)
//     }
// }

pub struct FileError {
    pub err: std::io::Error,
    pub file: PathBuf,
    pub node: Option<Node>,
}

impl Diagnostic for FileError {
    fn to_diag<'a>(self, sources: &'a SourceMap) -> Group<'a> {
        let snippet = self.node.map(|node| {
            let src = sources.get_idx(node.src).unwrap();
            Snippet::source(&src.contents)
                .path(src.path.as_os_str().to_str().unwrap())
                .annotation(AnnotationKind::Primary.span(node.range.into()))
        });

        Level::ERROR
            .primary_title(format!(
                "failed to load file {}: {}",
                self.file.display(),
                self.err
            ))
            .elements(snippet)
    }
}


pub struct ExpectedItem{

}