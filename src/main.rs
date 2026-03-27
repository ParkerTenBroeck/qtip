use std::path::Path;

use twinkle::{compiler::Compiler, source::SourceMap};

fn main() {
    let sm = SourceMap::new(|path| {
        let start: &Path = "./twinkle/src/".as_ref();
        std::fs::read_to_string(start.join(path))
    });
    let result = Compiler::compile(&sm);
    println!("{result}");
}
