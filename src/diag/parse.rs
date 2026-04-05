use proc_macros::Diagnostic;

use crate::{lex::Token, node::Node};

#[derive(Diagnostic)]
#[diag("Expected symbol found {found}")]
pub struct ExpectedSymbol<'a> {
    #[primary_node]
    pub node: Node,
    pub found: Token<'a>,
}

#[derive(Diagnostic)]
#[diag("Expected `;`, found {found}")]
pub struct ExpectedSemi<'a> {
    #[primary_node]
    #[label("help: add `;` here")]
    pub node: Node,

    pub found: Token<'a>,
    #[label("unexpected token")]
    pub found_node: Node,
}
