pub mod scoreboard;

use super::CompileError;
use crate::compiler::ast::serialiser::IToken;

use scoreboard::command_ast::CommandAST;
pub use scoreboard::Scoreboard;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type { Int, Float, Bool, Str, None }
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::None => "None",
            Type::Bool => "Bool",
            Type::Float => "Float",
            Type::Int => "Int",
            Type::Str => "Str"
        })
    }
}
pub trait Operator {
    fn get_priority(&self) -> u32;
    fn to_str(&self) -> &str;
    fn calc(&self, left:&Scoreboard, right:&IToken) -> Result<Vec<CommandAST>, CompileError>;
    fn get_type(&self, left:&Type, right:&Type) -> Option<Type>;
}
#[derive(Debug, Clone)]
pub enum Oper {
    Arithmetic(scoreboard::arithmetic_operation::Arithmetic),
    Logical(scoreboard::logical_operation::Logical),
    Comparison(scoreboard::comparison_operation::Comparison)
}
impl std::fmt::Display for Oper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Arithmetic(a) => a.to_str(),
            Self::Comparison(c) => c.to_str(),
            Self::Logical(l) => l.to_str()
        })
    }
}
impl Oper {
    pub fn get_priority(&self) -> u32 {
        match self {
            Oper::Arithmetic(o) => o.get_priority(),
            Oper::Comparison(o) => o.get_priority(),
            Oper::Logical(o) => o.get_priority(),
        }
    }
    pub fn to_str(&self) -> &str {
        match self {
            Oper::Arithmetic(o) => o.to_str(),
            Oper::Comparison(o) => o.to_str(),
            Oper::Logical(o) => o.to_str(),
        }
    }
}