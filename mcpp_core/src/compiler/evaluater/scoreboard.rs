use std::vec;

use rand::Rng;
use crate::compiler::CompileError;

use super::Operator;
use super::Types;
use super::FToken;

const NAMESPACE:&str = "MCPP.var";
const FLOAT_MAGNIFICATION:i32 = 1000;
pub const TEMP_ID_LEN:u32 = 32;

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

pub fn oper_to_str(operator:&Operator) -> &str {
    match operator {
        Operator::Add => "+",
        Operator::Rem => "-",
        Operator::Mul => "*",
        Operator::Div => "/",
        Operator::Sur => "%",
        Operator::Asn => "="
    }
}

pub fn generate_random_id(length:u32) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.random_range('a'..='z') as char)
        .collect::<String>()
}

impl Scoreboard {
    pub fn get_mcname(&self) -> String {
        format!("#{}{}{}", self.scope.join("."), if !self.scope.is_empty() {"."} else {""}, self.name)
    }
    
    fn pure_calc_score(&self, operator:&Operator, right:&Scoreboard) -> String {
        format!(
            "scoreboard players operation {} {} {}= {} {}",
            self.get_mcname(),
            NAMESPACE,
            oper_to_str(operator),
            right.get_mcname(),
            NAMESPACE
        )
    }
    fn pure_assign_score(&self, right:&Scoreboard) -> String {
        format!(
            "scoreboard players operation {} {} = {} {}",
            self.get_mcname(),
            NAMESPACE,
            right.get_mcname(),
            NAMESPACE
        )
    }
    
    fn pure_assign_num(&self, right:i32) -> String {
        format!("scoreboard players set {} {} {}", self.get_mcname(), NAMESPACE, right)
    }
    fn pure_calc_num(&self, operator:&Operator, right:i32) -> String {
        let rem_add = |given:&str| format!("scoreboard players {} {} {} {}", given, self.get_mcname(), NAMESPACE, right);
        match operator {
            Operator::Add => rem_add("add"),
            Operator::Rem => rem_add("remove"),
            Operator::Mul | Operator::Div | Operator::Sur => {
                let temp = super::get_const(&FToken::Int(right)).unwrap();
                format!(
                    "{}\n{}",
                    temp.0,
                    self.pure_calc_score(operator, &temp.1)
                )
            },
            Operator::Asn => self.pure_assign_num(right)
        }
    }
    
    fn fltfy(&self) -> String {
        self.pure_calc_num(&Operator::Mul, FLOAT_MAGNIFICATION)
    }
    fn intfy(&self) -> String {
        self.pure_calc_num(&Operator::Div, FLOAT_MAGNIFICATION)
    }

    pub fn calc(&self, operator:&Operator, right:&FToken) -> Result<String, CompileError> {
        match right {
            FToken::Int(i) => match self.datatype {
                Types::Int => Ok(self.pure_calc_num(operator, *i)),
                Types::Float => Ok(self.pure_calc_num(operator, *i * FLOAT_MAGNIFICATION)),
            },
            FToken::Flt(f) => match self.datatype {
                Types::Int => Ok(self.pure_calc_num(operator, f.floor() as i32)),
                Types::Float => Ok(self.pure_calc_num(operator, (*f * (FLOAT_MAGNIFICATION as f32)) as i32))
            },
            FToken::Scr(s) => {
                match (self.datatype, s.datatype) {
                    (Types::Int, Types::Int) => Ok(self.pure_calc_score(operator, s)),
                    (Types::Float, Types::Float) => match operator {
                        Operator::Mul => Ok([
                            self.pure_calc_score(operator, s),
                            self.intfy()
                        ].join("\n")),
                        Operator::Div => Ok([
                            self.fltfy(),
                            self.pure_calc_score(operator, s),
                        ].join("\n")),
                        _ => Ok(self.pure_calc_score(operator, s))
                    },
                    (Types::Int, Types::Float) => {
                        let type_adjusted = Scoreboard {
                            name: format!("TYPE_ADJUSTED_{}", generate_random_id(TEMP_ID_LEN)),
                            scope: vec!["TEMP".to_string()],
                            datatype: Types::Int
                        };
                        Ok([
                            type_adjusted.pure_assign_score(s),
                            type_adjusted.intfy(),
                            self.pure_calc_score(operator, &type_adjusted),
                            type_adjusted.free()
                        ].join("\n"))
                    },
                    (Types::Float, Types::Int) => {
                        let type_adjusted = Scoreboard {
                            name: format!("TYPE_ADJUSTED_{}", generate_random_id(TEMP_ID_LEN)),
                            scope: vec!["TEMP".to_string()],
                            datatype: Types::Float
                        };
                        Ok([
                            type_adjusted.pure_assign_score(s),
                            type_adjusted.fltfy(),
                            self.pure_calc_score(operator, &type_adjusted),
                            type_adjusted.free()
                        ].join("\n"))
                    }
                }
            },
            _ => Err(CompileError::InvalidRHS(right.clone()))
        }
    }

    pub fn assign(&self, right:&FToken) -> Result<String, CompileError> {
        match right {
            FToken::Int(i) => match self.datatype {
                Types::Int => Ok(self.pure_assign_num(*i)),
                Types::Float => Ok(self.pure_assign_num(i * FLOAT_MAGNIFICATION))
            },
            FToken::Flt(f) => match self.datatype {
                Types::Int => Ok(self.pure_assign_num(*f as i32)),
                Types::Float => Ok(self.pure_assign_num((*f * (FLOAT_MAGNIFICATION as f32)) as i32))
            },
            FToken::Scr(s) => {
                match self.datatype {
                    Types::Int => match s.datatype {
                        Types::Int => Ok(self.pure_assign_score(s)),
                        Types::Float => Ok(format!(
                            "{}\n{}",
                            self.pure_assign_score(s),
                            self.pure_calc_num(&Operator::Div, FLOAT_MAGNIFICATION)
                        ))
                    },
                    Types::Float => match s.datatype {
                        Types::Int => Ok(format!(
                            "{}\n{}",
                            self.pure_assign_score(s),
                            self.pure_calc_num(&Operator::Mul, FLOAT_MAGNIFICATION)
                        )),
                        Types::Float => Ok(self.pure_assign_score(s))
                    }
                }
            },
            _ => Err(CompileError::TheTokenIsntValue(right.clone()))
        }
    }
    pub fn free(&self) -> String {
        format!("scoreboard players reset {} {}", self.get_mcname(), NAMESPACE)
    }
}