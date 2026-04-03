
use proc_macros::Diagnostic;

use crate::{lex::LexError, node::Node};

#[derive(Diagnostic)]
#[diag("{msg}")]
pub struct LexerError {
    #[primary_node]
    pub node: Node,
    pub msg: LexError,
}