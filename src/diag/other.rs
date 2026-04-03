use std::path::PathBuf;

use proc_macros::Diagnostic;

use crate::node::Node;


#[derive(Diagnostic)]
#[diag("Failed to load file `{$file.display()}` {err}")]
pub struct FileError {
    pub err: std::io::Error,
    pub file: PathBuf,
    #[primary_node]
    pub node: Option<Node>,
}
