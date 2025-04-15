pub mod compiler;
use compiler::CompileError;
pub use compiler::{evaluater, tokeniser};

pub fn evaluate(input:&str) -> Result<String, CompileError> {
    let mut compiler = compiler::Compiler::new();
    Ok(
        evaluater::evaluate(
            &mut compiler,
            &tokeniser::tokenize(input.to_string())
        )?.join("\n")
    )
}