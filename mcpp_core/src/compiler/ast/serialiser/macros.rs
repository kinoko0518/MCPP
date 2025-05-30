use crate::{compiler::ast::{FToken, SyntaxError, Tuple}, evaluater::scoreboard::command_ast::CommandAST};

pub fn solve_native(arg:Tuple) -> Result<CommandAST, SyntaxError> {
    if arg.inside.len() == 0 {
        let inside = arg.inside.get(0).unwrap();
        if let Some(FToken::Str(s)) = inside.formula_tokens.get(0) {
            Ok(CommandAST::Native(s.clone()))
        } else {
            Err(SyntaxError::ArgumentCountMismatch)
        }
    } else {
        Err(SyntaxError::ArgumentCountMismatch)
    }
}