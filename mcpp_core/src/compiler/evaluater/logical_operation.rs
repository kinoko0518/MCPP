use crate::compiler::CompileError;

use super::{arithmetic_operation::Arithmetic, comparison_operation::Comparison, FToken, Operator, Scoreboard, Types};

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
}
impl Logical {
    pub fn logicalc(&self, left:&Scoreboard, right:&FToken) -> Result<String, CompileError> {
        if let (Types::Bool, Some(Types::Bool)) = (left.datatype, right.get_datatype_or()) {
            match self {
                Logical::And => Arithmetic::Mul.calc(&left, right),
                Logical::Or  => Ok(format!(
                    "{}\n{}",
                    Arithmetic::Add.calc(&left, right)?,
                    Comparison::Neq.compare_to_get_boolean(&left, &FToken::Int(0))?
                )),
                _ => todo!("! operator is still on development, sorry!")
            }
        } else {
            Err(CompileError::CalcationBetweenUnableTypes(left.datatype, match right.get_datatype_or() {
                Some(s) => s,
                None => {return Err(CompileError::TheTokenIsntValue(right.clone()));}
            }))
        }
    }
}