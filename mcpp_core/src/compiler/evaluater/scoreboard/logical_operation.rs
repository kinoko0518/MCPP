use super::super::Operator;
use super::command_ast::{FormulaConstructer, ScoreAST};
use super::Scoreboard;
use crate::compiler::CompileError;
use crate::compiler::FToken;
use crate::evaluater::{Oper, Types};

#[derive(Debug, Clone)]
pub enum Logical { And, Or, Not }

impl Operator for Logical {
    fn get_priority(&self) -> u32 {
        match self {
            Self::And | Self::Or => 1,
            Self::Not => 0
        }
    }
    fn to_str(&self) -> &str {
        match self {
            Self::And => "&",
            Self::Or  => "|",
            Self::Not => "!"
        }
    }
    fn calc(&self, left:&Scoreboard, right:&FToken) -> Result<Vec<ScoreAST>, CompileError> {
        match right {
            FToken::Scr(s) => self.logicalc_score(left, s),
            FToken::Bln(b) => self.logicalc_bool(left, *b),
            FToken::Int(_) => Err(
                CompileError::UndefinedOperation(left.datatype, Oper::Logical(self.clone()), Types::Int)
            ),
            FToken::Flt(_) => Err(
                CompileError::UndefinedOperation(left.datatype, Oper::Logical(self.clone()), Types::Float)
            ),
            _ => Err(CompileError::TheTokenIsntValue(right.clone()))
        }
    }
}
impl Logical {
    fn logicalc_score(&self, left:&Scoreboard, right:&Scoreboard) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Logical(self.clone()),
            right.datatype.clone()
        );
        match (left.datatype, right.datatype) {
            (Types::Bool, Types::Bool) => match self {
                Logical::And => Ok(
                    f_constract
                        .calc_score(left, "*=".to_string(), right)
                        .build()
                ),
                Logical::Or => Ok(
                    f_constract
                        .calc_score(left, "+=".to_string(), right)
                        .validate_bool(left)
                        .build()
                ),
                Logical::Not => todo!()
            },
            _ => Err(undefined_operation_occured)
        }
    }
    fn logicalc_bool(&self, left:&Scoreboard, right:bool) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Logical(self.clone()),
            Types::Bool
        );
        match left.datatype {
            Types::Bool => match self {
                Logical::And => match right {
                    true => Ok(Vec::new()),
                    false => Ok(
                        f_constract
                            .assign_num(left, 0)
                            .build()
                        )
                },
                Logical::Or => match right {
                    true => Ok(
                        f_constract
                            .assign_num(left, 1)
                            .build()
                    ),
                    false => Ok(Vec::new())
                },
                _ => Err(undefined_operation_occured)
            }
            _ => Err(undefined_operation_occured)
        }
    }
}