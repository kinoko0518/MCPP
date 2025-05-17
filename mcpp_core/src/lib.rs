pub mod compiler;
use compiler::CompileError;
pub use compiler::{evaluater, tokeniser};
use crate::compiler::evaluater::scoreboard::command_ast::Serialise;

pub fn evaluate(input:&str) -> Result<String, CompileError> {
    let mut compiler = compiler::Compiler::new();
    Ok(
        evaluater::evaluate(
            &mut compiler,
            &tokeniser::tokenize(input.to_string())
        )?
            .iter()
            .map(|ast| ast.clone().serialise()).collect::<Vec<String>>()
            .join("\n")
    )
}