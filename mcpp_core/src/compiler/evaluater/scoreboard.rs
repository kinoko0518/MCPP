pub mod arithmetic_operation;
pub mod comparison_operation;
pub mod logical_operation;
pub mod command_ast;

use command_ast::{FormulaConstructer, CommandAST};
use rand::Rng;
use super::Type;
use super::CompileError;
use crate::compiler::ast::serialiser::IToken;

pub const NAMESPACE:&str = "MCPP.var";
pub const FLOAT_MAGNIFICATION:i32 = 1000;
pub const TEMP_ID_LEN:u32 = 16;

#[derive(Debug, Clone)]
pub struct Scoreboard {
    pub name : String,
    pub scope : Vec<String>,
    pub datatype : Type
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
    pub fn assign(&self, right:&IToken) -> Result<Vec<CommandAST>, CompileError> {
        let mut f_construct = FormulaConstructer::new();
        match right {
            IToken::Int(i) => match self.datatype {
                Type::Int => Ok(
                    f_construct
                        .assign_num(&self, *i)
                        .build()
                ),
                Type::Float => Ok(
                    f_construct
                        .assign_num(&self, *i * FLOAT_MAGNIFICATION)
                        .build()
                ),
                _ => Err(CompileError::InvalidRHS(right.clone()))
            },
            IToken::Flt(f) => match self.datatype {
                Type::Int => Ok(
                    f_construct
                        .assign_num(&self, *f as i32)
                        .build()
                ),
                Type::Float => Ok(
                    f_construct
                        .assign_num(&self, (*f * (FLOAT_MAGNIFICATION as f32)) as i32)
                        .build()
                ),
                _ => Err(CompileError::InvalidRHS(right.clone()))
            },
            IToken::Scr(s) => {
                match self.datatype {
                    Type::Int => match s.datatype {
                        Type::Int => Ok(
                            f_construct
                                .assign_score(&self, s)
                                .build()
                        ),
                        Type::Float => Ok(
                            f_construct
                                .assign_score(&self, s)
                                .intify(&self)
                                .build()
                        ),
                        _ => Err(CompileError::InvalidRHS(right.clone()))
                    },
                    Type::Float => match s.datatype {
                        Type::Int => Ok(
                            f_construct
                                .assign_score(&self, s)
                                .fltify(&self)
                                .build()
                        ),
                        Type::Float => Ok(
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
    pub fn free(&self) -> Vec<CommandAST> {
        FormulaConstructer::new().free(&self).build()
    }
}
pub fn get_type_adjusted_temp(datatype:Type) -> Scoreboard {
    Scoreboard {
        name: format!("CALC_TYPE_ADJUSTED_{}", generate_random_id(16)),
        scope: vec!["TEMP".to_string()],
        datatype: datatype
    }
}
pub fn get_calc_temp(datatype:Type) -> Scoreboard {
    Scoreboard {
        name: format!("CALC_TEMP_{}", generate_random_id(16)),
        scope: vec!["TEMP".to_string()],
        datatype: datatype
    }
}
pub fn get_calc_result_temp(datatype:Type) -> Scoreboard {
    Scoreboard {
        name: format!("CALC_RESULT_{}", generate_random_id(16)),
        scope: vec!["TEMP".to_string()],
        datatype: datatype
    }
}
pub fn generate_random_id(length:u32) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.random_range('a'..='z') as char)
        .collect::<String>()
}