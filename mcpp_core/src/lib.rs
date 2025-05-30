pub mod compiler;
use compiler::{ast::{serialiser::{MCFunction}, syntax_analyser}, CompileError, Compiler, Token};
use crate::compiler::ast::serialiser::MCFunctionizable;
pub use compiler::{evaluater, tokeniser};

pub fn compile(input:&str) -> Result<Vec<MCFunction>, CompileError> {
    let mut inside = vec![Token::LBrace];
    inside.extend(tokeniser::tokenize(input.to_string()));
    inside.extend(vec![Token::RBrace]);
    let mut analyser = syntax_analyser::SyntaxAnalyser::from(inside);
    let mut compiler = Compiler::from("MCPP");
    Ok(vec![match analyser.get_block() {
        Ok(o) => o,
        Err(e) => Err(CompileError::ASyntaxErrorOccured(e))?
    }.mcfunctionate(&mut compiler)?])
}