use std::num::NonZeroU32;

use crate::{source::SrcIdx, span::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    pub range: Span,
    pub src: SrcIdx,
    pub parent: Option<ParentIdx>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParentIdx(NonZeroU32);
