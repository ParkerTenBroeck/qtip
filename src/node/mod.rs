use std::num::NonZeroU32;

use crate::{source::SrcIdx, span::Span};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Node {
    pub span: Span,
    pub src: SrcIdx,
    pub parent: Option<ParentIdx>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}..{}",
            self.src.idx(),
            self.span.start,
            self.span.end
        )
    }
}

impl Node {
    pub fn after(self) -> Self {
        Self {
            span: Span::new(self.span.end, self.span.end),
            src: self.src,
            parent: None,
        }
    }
    
    pub fn before(&self) -> Node {
        Self { 
            span: Span::new(self.span.start, self.span.start), 
            src: self.src,
            parent: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParentIdx(NonZeroU32);
