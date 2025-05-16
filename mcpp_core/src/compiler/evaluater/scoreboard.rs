use std::vec;

use crate::compiler::CompileError;

use super::arithmetic_operation::Arithmetic;
use super::Types;
use super::FToken;

pub const NAMESPACE:&str = "MCPP.var";
pub const FLOAT_MAGNIFICATION:i32 = 1000;
pub const TEMP_ID_LEN:u32 = 16;

pub trait Constnisable {
    fn get_const(&self) -> (String, Scoreboard);
}
impl Constnisable for i32 {
    fn get_const(&self) -> (String, Scoreboard) {
        let score = Scoreboard {
            name: self.to_string(),
            scope: vec!["CONSTANT".to_string()],
            datatype: Types::Int
        };
        (score.pure_assign_num(*self), score)
    }
}
impl Constnisable for f32 {
    fn get_const(&self) -> (String, Scoreboard) {
        let score = Scoreboard {
            name: self.to_string(),
            scope: vec!["CONSTANT".to_string()],
            datatype: Types::Float
        };
        (score.pure_assign_num((*self * FLOAT_MAGNIFICATION as f32) as i32), score)
    }
}

#[derive(Debug, Clone)]
pub struct Scoreboard {
    pub name : String,
    pub scope : Vec<String>,
    pub datatype : Types
}
impl std::fmt::Display for Scoreboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", self.get_mcname(), self.datatype)
    }
}

impl Scoreboard {
    pub fn get_mcname(&self) -> String {
        format!("#{}{}{}", self.scope.join("."), if !self.scope.is_empty() {"."} else {""}, self.name)
    }
    
    pub fn pure_assign_score(&self, right:&Scoreboard) -> String {
        format!(
            "scoreboard players operation {} {} = {} {}",
            self.get_mcname(),
            NAMESPACE,
            right.get_mcname(),
            NAMESPACE
        )
    }
    pub fn pure_assign_num(&self, right:i32) -> String {
        format!("scoreboard players set {} {} {}", self.get_mcname(), NAMESPACE, right)
    }
    
    pub fn fltfy(&self) -> String {
        Arithmetic::Mul.pure_calc_num(self, FLOAT_MAGNIFICATION)
    }
    pub fn intfy(&self) -> String {
        Arithmetic::Div.pure_calc_num(self, FLOAT_MAGNIFICATION)
    }

    pub fn assign(&self, right:&FToken) -> Result<String, CompileError> {
        match right {
            FToken::Int(i) => match self.datatype {
                Types::Int => Ok(self.pure_assign_num(*i)),
                Types::Float => Ok(self.pure_assign_num(i * FLOAT_MAGNIFICATION)),
                _ => Err(CompileError::InvalidRHS(right.clone()))
            },
            FToken::Flt(f) => match self.datatype {
                Types::Int => Ok(self.pure_assign_num(*f as i32)),
                Types::Float => Ok(self.pure_assign_num((*f * (FLOAT_MAGNIFICATION as f32)) as i32)),
                _ => Err(CompileError::InvalidRHS(right.clone()))
            },
            FToken::Scr(s) => {
                match self.datatype {
                    Types::Int => match s.datatype {
                        Types::Int => Ok(self.pure_assign_score(s)),
                        Types::Float => Ok(format!(
                            "{}\n{}",
                            self.pure_assign_score(s),
                            &Arithmetic::Div.pure_calc_num(self, FLOAT_MAGNIFICATION)
                        )),
                        _ => Err(CompileError::InvalidRHS(right.clone()))
                    },
                    Types::Float => match s.datatype {
                        Types::Int => Ok(format!(
                            "{}\n{}",
                            self.pure_assign_score(s),
                            &Arithmetic::Mul.pure_calc_num(self, FLOAT_MAGNIFICATION)
                        )),
                        Types::Float => Ok(self.pure_assign_score(s)),
                        _ => Err(CompileError::InvalidRHS(right.clone()))
                    },
                    _ => Err(CompileError::InvalidRHS(right.clone()))
                }
            },
            _ => Err(CompileError::TheTokenIsntValue(right.clone()))
        }
    }
    pub fn free(&self) -> String {
        format!("scoreboard players reset {} {}", self.get_mcname(), NAMESPACE)
    }
}