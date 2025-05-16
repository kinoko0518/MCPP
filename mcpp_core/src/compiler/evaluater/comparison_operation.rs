use crate::compiler::CompileError;

use super::{get_temp_score, FToken, Operator, Scoreboard, Types};
use super::scoreboard::NAMESPACE;

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
}
impl Comparison {
    fn pure_compare_score(&self, left:&Scoreboard, right:&Scoreboard) -> String {
        if let &Comparison::Neq = self {
            format!(
                "unless score {} {} {} {} {}",
                left.get_mcname(),
                NAMESPACE,
                &Comparison::Eq.to_str(),
                right.get_mcname(),
                NAMESPACE
            )
        } else {
            format!(
                "if score {} {} {} {} {}",
                left.get_mcname(),
                NAMESPACE,
                self.to_str(),
                right.get_mcname(),
                NAMESPACE
            )
        }
    }
    pub fn condition_to_boolean(contain_to:&Scoreboard, condition:&str) -> Result<String, CompileError> {
        let temp = get_temp_score(Types::Bool);
        Ok(format!(
            "{}\nexecute {} run {}\n{}",
            temp.pure_assign_num(0),
            condition,
            temp.pure_assign_num(1),
            contain_to.assign(&FToken::Scr(temp))?
        ))
    }
    pub fn compare_to_get_boolean(&self, left:&Scoreboard, right:&FToken) -> Result<String, CompileError> {
        let r_type = match right.get_datatype_or() {
            Some(s) => s,
            None => return Err(CompileError::TheTokenIsntValue(right.clone()))
        };
        if left.datatype == r_type {
            if let FToken::Scr(s) = right {
                return Ok(Self::condition_to_boolean(
                    left, &self.pure_compare_score(left, s)
                )?);
            }
        }
        match (left.datatype, r_type) {
            (Types::Int, Types::Float) => {
                let temp = get_temp_score(Types::Int);
                Ok([
                    temp.assign(right)?,
                    self.compare_to_get_boolean(left, &FToken::Scr(temp))?
                ].join("\n"))
            },
            (Types::Float, Types::Int) => {
                let temp = get_temp_score(Types::Float);
                Ok([
                    temp.assign(right)?,
                    self.compare_to_get_boolean(left, &FToken::Scr(temp))?
                ].join("\n"))
            },
            _ => Err(CompileError::CalcationBetweenUnableTypes(left.datatype, r_type))
        }
    }
}