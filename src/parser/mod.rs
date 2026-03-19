use crate::{lex::Lexer, parser::ast::Program};

pub mod ast;



pub struct Parser<'a>{
    lexer: Lexer<'a>
}

impl<'a> Parser<'a>{
    pub fn parse(&mut self) -> Program<'a>{
        todo!()
    } 
}