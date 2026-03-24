use qtip::{lex::Lexer, parser::Parser};

fn main() {
    let program = "
        fn main() -> i32 {
            1+2
        }
    ";

    let program = Parser::new(Lexer::new(program)).parse();
    println!("{:#?}", program)
}
