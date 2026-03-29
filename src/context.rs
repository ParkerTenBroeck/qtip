use std::{cell::RefCell, rc::Rc};

use crate::{
    diag::{Diagnostic, Diagnostics},
    source::SourceMap,
};

#[derive(Clone)]
pub struct Context<'a> {
    pub sources: &'a SourceMap,
    pub diag: Rc<RefCell<Diagnostics<'a>>>,
}

impl<'a> Context<'a> {
    pub fn new(sources: &'a SourceMap) -> Self {
        Self {
            sources,
            diag: Default::default(),
        }
    }

    pub fn report(&self, diag: impl Diagnostic<'a>) {
        self.diag.borrow_mut().report(self.sources, diag);
    }
}
