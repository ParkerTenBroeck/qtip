use annotate_snippets::*;

#[derive(Default)]
pub struct Diagnostics<'a> {
    diags: Vec<Group<'a>>,
}

impl<'a> std::fmt::Display for Diagnostics<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&annotate_snippets::Renderer::styled().render(&self.diags))
    }
}

impl<'a> Diagnostics<'a> {
    pub fn new() -> Self {
        Diagnostics { diags: vec![] }
    }

    pub fn report(&mut self, report: impl Into<Group<'a>>) {
        self.diags.push(report.into());
    }
}
