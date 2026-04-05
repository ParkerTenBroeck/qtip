use std::num::NonZeroU32;

use crate::{source::SrcIdx, span::Span};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    pub span: Span,
    pub src: SrcIdx,
    pub parent: Option<ParentIdx>,
}

impl Node {
    pub fn after(self) -> Self {
        Self {
            span: Span::new(self.span.end, self.span.end),
            src: self.src,
            parent: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParentIdx(NonZeroU32);
