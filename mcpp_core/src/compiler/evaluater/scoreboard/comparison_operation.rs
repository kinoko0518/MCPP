use super::super::Operator;
use super::command_ast::{FormulaConstructer, ScoreAST};
use super::{get_type_adjusted_temp, Scoreboard, FLOAT_MAGNIFICATION};
use crate::compiler::CompileError;
use crate::compiler::FToken;
use crate::evaluater::{Oper, Types};

#[derive(Debug, Clone)]
pub enum Comparison { Gt, Ge, Lt, Le, Eq, Neq }

impl Operator for Comparison {
    fn get_priority(&self) -> u32 {
        0
    }
    fn to_str(&self) -> &str {
        match self {
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Eq => "==",
            Self::Neq => "!="
        }
    }
    fn calc(&self, left:&Scoreboard, right:&FToken) -> Result<Vec<ScoreAST>, CompileError> {
        match right {
            FToken::Scr(s) => self.compare_score(left, s),
            FToken::Int(i) => self.compare_int(left, *i),
            FToken::Flt(f) => self.compare_float(left, *f),
            FToken::Bln(b) => self.compare_bool(left, *b),
            _ => Err(CompileError::TheTokenIsntValue(right.clone()))
        }
    }
}
impl Comparison {
    fn compare_score(&self, left:&Scoreboard, right:&Scoreboard) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            right.datatype.clone()
        );
        let cmp = self.to_str().to_string();
        match (left.datatype, right.datatype) {
            (Types::Int, Types::Int) | (Types::Float, Types::Float) | (Types::Bool, Types::Bool) => Ok(
                f_constract
                    .boolify_score_comparison(
                        left, cmp, right
                    )
                    .build()
            ),
            (Types::Float, Types::Int) => Ok({
                let adjusted = get_type_adjusted_temp(Types::Float);
                f_constract
                    .assign_score(&adjusted, right)
                    .fltify(&adjusted)
                    .boolify_score_comparison(left,cmp, &adjusted)
                    .free(&adjusted)
                    .build()
            }),
            (Types::Int, Types::Float) => Ok({
                let adjusted = get_type_adjusted_temp(Types::Float);
                f_constract
                    .assign_score(&adjusted, left)
                    .fltify(&adjusted)
                    .boolify_score_comparison(&adjusted,cmp, right)
                    .free(&adjusted)
                    .build()
            }),
            _ => Err(undefined_operation_occured)
        }
    }
    fn compare_int(&self, left:&Scoreboard, right:i32) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            Types::Int
        );
        let cmp = self.to_str().to_string();
        match left.datatype {
            Types::Int => Ok(
                f_constract
                    .boolify_num_comparison(left, cmp, right)
                    .build()
            ),
            Types::Float => Ok(
                f_constract
                    .boolify_num_comparison(left, cmp, right * FLOAT_MAGNIFICATION)
                    .build()
            ),
            _ => Err(undefined_operation_occured)
        }
    }
    fn compare_float(&self, left:&Scoreboard, right:f32) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            Types::Int
        );
        let scaled = (right * FLOAT_MAGNIFICATION as f32).floor() as i32;
        let cmp = self.to_str().to_string();
        match left.datatype {
            Types::Int => {
                let adjusted = get_type_adjusted_temp(Types::Float);
                Ok(
                    f_constract
                        .assign_score(&adjusted, left)
                        .fltify(&adjusted)
                        .boolify_num_comparison(left, cmp, scaled)
                        .free(&adjusted)
                        .build()
                )
            },
            Types::Float => Ok(
                f_constract
                    .boolify_num_comparison(left, cmp, scaled)
                    .build()
            ),
            _ => Err(undefined_operation_occured)
        }
    }
    fn compare_bool(&self, left:&Scoreboard, right:bool) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            Types::Int
        );
        match left.datatype {
            Types::Bool => match self {
                Self::Eq => Ok(
                    f_constract
                        .boolify_num_comparison(left, match right {
                            true => Self::Neq.to_str(),
                            false => Self::Eq.to_str(),
                        }.to_string(), 0)
                        .build()
                ),
                Self::Neq => Ok(
                    f_constract
                        .boolify_num_comparison(left, match right {
                            true => Self::Eq.to_str(),
                            false => Self::Neq.to_str(),
                        }.to_string(), 0)
                        .build()
                ),
                _ => Err(undefined_operation_occured)
            },
            _ => Err(undefined_operation_occured)
        }
    }
}