use super::super::Operator;
use super::command_ast::{FormulaConstructer, CommandAST};
use super::{get_type_adjusted_temp, Scoreboard, FLOAT_MAGNIFICATION};
use crate::compiler::CompileError;
use crate::evaluater::{Oper, Type};
use crate::compiler::ast::serialiser::IToken;

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
    fn calc(&self, left:&Scoreboard, right:&IToken) -> Result<Vec<CommandAST>, CompileError> {
        match right {
            IToken::Scr(s) => self.compare_score(left, s),
            IToken::Int(i) => self.compare_int(left, *i),
            IToken::Flt(f) => self.compare_float(left, *f),
            IToken::Bln(b) => self.compare_bool(left, *b),
            _ => Err(CompileError::TheTokenIsntValue(right.clone()))
        }
    }
    fn get_type(&self, left:&Type, right:&Type) -> Option<Type> {
        if left == right {
            return Some(Type::Bool)
        }
        match (left, right) {
            (Type::Int, Type::Float) => Some(Type::Bool),
            (Type::Float, Type::Int) => Some(Type::Bool),
            _ => None
        }
    }
}
impl Comparison {
    fn compare_score(&self, left:&Scoreboard, right:&Scoreboard) -> Result<Vec<CommandAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            right.datatype.clone()
        );
        let cmp = self.to_str().to_string();
        match (left.datatype, right.datatype) {
            (Type::Int, Type::Int) | (Type::Float, Type::Float) | (Type::Bool, Type::Bool) => Ok(
                f_constract
                    .boolify_score_comparison(
                        left, cmp, right
                    )
                    .build()
            ),
            (Type::Float, Type::Int) => Ok({
                let adjusted = get_type_adjusted_temp(Type::Float);
                f_constract
                    .assign_score(&adjusted, right)
                    .fltify(&adjusted)
                    .boolify_score_comparison(left,cmp, &adjusted)
                    .free(&adjusted)
                    .build()
            }),
            (Type::Int, Type::Float) => Ok({
                let adjusted = get_type_adjusted_temp(Type::Float);
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
    fn compare_int(&self, left:&Scoreboard, right:i32) -> Result<Vec<CommandAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            Type::Int
        );
        let cmp = self.to_str().to_string();
        match left.datatype {
            Type::Int => Ok(
                f_constract
                    .boolify_num_comparison(left, cmp, right)
                    .build()
            ),
            Type::Float => Ok(
                f_constract
                    .boolify_num_comparison(left, cmp, right * FLOAT_MAGNIFICATION)
                    .build()
            ),
            _ => Err(undefined_operation_occured)
        }
    }
    fn compare_float(&self, left:&Scoreboard, right:f32) -> Result<Vec<CommandAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            Type::Int
        );
        let scaled = (right * FLOAT_MAGNIFICATION as f32).floor() as i32;
        let cmp = self.to_str().to_string();
        match left.datatype {
            Type::Int => {
                let adjusted = get_type_adjusted_temp(Type::Float);
                Ok(
                    f_constract
                        .assign_score(&adjusted, left)
                        .fltify(&adjusted)
                        .boolify_num_comparison(left, cmp, scaled)
                        .free(&adjusted)
                        .build()
                )
            },
            Type::Float => Ok(
                f_constract
                    .boolify_num_comparison(left, cmp, scaled)
                    .build()
            ),
            _ => Err(undefined_operation_occured)
        }
    }
    fn compare_bool(&self, left:&Scoreboard, right:bool) -> Result<Vec<CommandAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Comparison(self.clone()),
            Type::Int
        );
        match left.datatype {
            Type::Bool => match self {
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