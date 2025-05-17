pub mod arithmetic_operation;
pub mod comparison_operation;
pub mod logical_operation;
pub mod command_ast;

use command_ast::{FormulaConstructer, ScoreAST};
use rand::Rng;
use super::Types;
use super::CompileError;
use super::FToken;

pub const NAMESPACE:&str = "MCPP.var";
pub const FLOAT_MAGNIFICATION:i32 = 1000;
pub const TEMP_ID_LEN:u32 = 16;

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
    pub fn assign(&self, right:&FToken) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_construct = FormulaConstructer::new();
        match right {
            FToken::Int(i) => match self.datatype {
                Types::Int => Ok(
                    f_construct
                        .assign_num(&self, *i)
                        .build()
                ),
                Types::Float => Ok(
                    f_construct
                        .assign_num(&self, *i * FLOAT_MAGNIFICATION)
                        .build()
                ),
                _ => Err(CompileError::InvalidRHS(right.clone()))
            },
            FToken::Flt(f) => match self.datatype {
                Types::Int => Ok(
                    f_construct
                        .assign_num(&self, *f as i32)
                        .build()
                ),
                Types::Float => Ok(
                    f_construct
                        .assign_num(&self, (*f * (FLOAT_MAGNIFICATION as f32)) as i32)
                        .build()
                ),
                _ => Err(CompileError::InvalidRHS(right.clone()))
            },
            FToken::Scr(s) => {
                match self.datatype {
                    Types::Int => match s.datatype {
                        Types::Int => Ok(
                            f_construct
                                .assign_score(&self, s)
                                .build()
                        ),
                        Types::Float => Ok(
                            f_construct
                                .assign_score(&self, s)
                                .intify(&self)
                                .build()
                        ),
                        _ => Err(CompileError::InvalidRHS(right.clone()))
                    },
                    Types::Float => match s.datatype {
                        Types::Int => Ok(
                            f_construct
                                .assign_score(&self, s)
                                .fltify(&self)
                                .build()
                        ),
                        Types::Float => Ok(
                            f_construct
                                .assign_score(&self, s)
                                .build()
                        ),
                        _ => Err(CompileError::InvalidRHS(right.clone()))
                    },
                    _ => Err(CompileError::InvalidRHS(right.clone()))
                }
            },
            _ => Err(CompileError::TheTokenIsntValue(right.clone()))
        }
    }
    pub fn free(&self) -> Vec<ScoreAST> {
        FormulaConstructer::new().free(&self).build()
    }
}

pub fn generate_random_id(length:u32) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.random_range('a'..='z') as char)
        .collect::<String>()
}
pub fn get_calc_temp(datatype:Types) -> Scoreboard {
    Scoreboard {
        name: format!("CALC_TEMP_{}", generate_random_id(16)),
        scope: vec!["TEMP".to_string()],
        datatype: datatype
    }
}
pub fn get_type_adjusted_temp(datatype:Types) -> Scoreboard {
    Scoreboard {
        name: format!("CALC_TYPE_ADJUSTED_{}", generate_random_id(16)),
        scope: vec!["TEMP".to_string()],
        datatype: datatype
    }
}