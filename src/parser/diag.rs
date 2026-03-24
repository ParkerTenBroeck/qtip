use annotate_snippets::*;

pub struct Diagnostics<'a> {
    diags: Vec<Annotation<'a>>,
}

impl<'a> Diagnostics<'a> {
    pub fn new() -> Self {
        Diagnostics { diags: vec![] }
    }

    pub fn report(&mut self) {}
}
