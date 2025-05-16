use crate::evaluater::scoreboard::Constnisable;
use super::CompileError;

use super::{FToken, Operator, Scoreboard, Types};

use super::scoreboard::NAMESPACE;
use super::scoreboard::FLOAT_MAGNIFICATION;
use super::scoreboard::TEMP_ID_LEN;

#[derive(Debug, Clone)]
pub enum Arithmetic { Add, Rem, Mul, Div, Sur }

impl Operator for Arithmetic {
    fn get_priority(&self) -> u32 {
        match self {
            Arithmetic::Mul | Arithmetic::Div | Arithmetic::Sur => 3,
            Arithmetic::Add | Arithmetic::Rem => 2,
        }
    }
    fn to_str(&self) -> &str {
        match self {
            Arithmetic::Add => "+",
            Arithmetic::Rem => "-",
            Arithmetic::Mul => "*",
            Arithmetic::Div => "/",
            Arithmetic::Sur => "%"
        }
    }
}
impl Arithmetic {
    pub fn pure_calc_score(&self, left:&Scoreboard, right:&Scoreboard) -> String {
        format!(
            "scoreboard players operation {} {} {}= {} {}",
            left.get_mcname(),
            NAMESPACE,
            self.to_str(),
            right.get_mcname(),
            NAMESPACE
        )
    }
    pub fn pure_calc_num(&self, left:&Scoreboard, right:i32) -> String {
        let rem_add = |given:&str| format!(
            "scoreboard players {} {} {} {}",
            given,
            left.get_mcname(),
            NAMESPACE,
            right
        );
        match self {
            Self::Add => rem_add("add"),
            Self::Rem => rem_add("remove"),
            Self::Mul | Self::Div | Self::Sur => {
                let temp = right.get_const();
                format!(
                    "{}\n{}",
                    temp.0,
                    self.pure_calc_score(left, &temp.1)
                )
            }
        }
    }
    pub fn calc(&self, left:&Scoreboard, right:&FToken) -> Result<String, CompileError> {
        if let FToken::Scr(s) = right.clone() {
            self.calc_score_and_score(left, &s)
        } else {
            self.calc_score_and_literal(left, right)
        }
    }
    fn calc_score_and_score(&self, left:&Scoreboard, right:&Scoreboard) -> Result<String, CompileError> {
        match (left.datatype, right.datatype) {
            (Types::Int, Types::Int) => Ok(self.pure_calc_score(left, right)),
            (Types::Float, Types::Float) => match self {
                &Arithmetic::Mul => Ok([
                    self.pure_calc_score(left, right),
                    left.intfy()
                ].join("\n")),
                &Arithmetic::Div => Ok([
                    left.fltfy(),
                    self.pure_calc_score(left, right),
                ].join("\n")),
                _ => Ok(self.pure_calc_score(left, right))
            },
            (Types::Int, Types::Float) => {
                let type_adjusted = Scoreboard {
                    name: format!("TYPE_ADJUSTED_{}", super::generate_random_id(TEMP_ID_LEN)),
                    scope: vec!["TEMP".to_string()],
                    datatype: Types::Int
                };
                Ok([
                    type_adjusted.pure_assign_score(right),
                    type_adjusted.intfy(),
                    self.pure_calc_score(left, &type_adjusted),
                    type_adjusted.free()
                ].join("\n"))
            },
            (Types::Float, Types::Int) => {
                let type_adjusted = Scoreboard {
                    name: format!("TYPE_ADJUSTED_{}", super::generate_random_id(TEMP_ID_LEN)),
                    scope: vec!["TEMP".to_string()],
                    datatype: Types::Float
                };
                Ok([
                    type_adjusted.pure_assign_score(right),
                    type_adjusted.fltfy(),
                    self.pure_calc_score(left, &type_adjusted),
                    type_adjusted.free()
                ].join("\n"))
            },
            _ => Err(CompileError::CalcationBetweenUnableTypes(left.datatype, right.datatype))
        }
    }
    fn calc_score_and_literal(&self, left:&Scoreboard, right:&FToken) -> Result<String, CompileError> {
        match right {
            FToken::Int(i) => match left.datatype {
                Types::Int => Ok(self.pure_calc_num(left, *i)),
                Types::Float => match self {
                    Arithmetic::Mul | Arithmetic::Div => Ok(self.pure_calc_num(left, *i)),
                    _ => Ok(self.pure_calc_num(left, *i * FLOAT_MAGNIFICATION))
                }
                _ => Err(CompileError::CalcationBetweenUnableTypes(Types::Int, left.datatype))
            },
            FToken::Flt(f) => match left.datatype {
                Types::Int => Ok(self.pure_calc_num(left, f.floor() as i32)),
                Types::Float => {
                    let calc_with_scaled = || self.pure_calc_num(left, (*f * (FLOAT_MAGNIFICATION as f32)) as i32);
                    match self {
                        Arithmetic::Mul => Ok([
                            calc_with_scaled(),
                            left.intfy()
                        ].join("\n")),
                        Arithmetic::Div => Ok([
                            left.fltfy(),
                            calc_with_scaled()
                        ].join("\n")),
                        _ => Ok(calc_with_scaled())
                    }
                },
                _ => Err(CompileError::CalcationBetweenUnableTypes(Types::Float, left.datatype))
            }
            _ => Err(CompileError::CalcationBetweenUnableTypes(left.datatype, match right.get_datatype_or() {
                Some(s) => s,
                None => {return Err(CompileError::TheTokenIsntValue(right.clone()));}
            }))
        }
    }
}