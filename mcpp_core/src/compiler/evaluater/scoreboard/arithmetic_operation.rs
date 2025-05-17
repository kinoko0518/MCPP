use super::super::Operator;
use super::command_ast::{FormulaConstructer, ScoreAST};
use super::{get_type_adjusted_temp, Scoreboard, FLOAT_MAGNIFICATION};
use crate::compiler::CompileError;
use crate::compiler::FToken;
use crate::evaluater::{Oper, Types};

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
    fn calc(&self, left:&Scoreboard, right:&FToken) -> Result<Vec<ScoreAST>, CompileError> {
        match right {
            FToken::Scr(s) => self.calc_score(left, s),
            FToken::Int(i) => self.calc_int(left, *i),
            FToken::Flt(f) => self.calc_float(left, *f),
            FToken::Bln(_) => Err(CompileError::UndefinedOperation(left.datatype, Oper::Arithmetic(self.clone()), Types::Bool)),
            _ => Err(CompileError::TheTokenIsntValue(right.clone()))
        }
    }
}
impl Arithmetic {
    fn calc_score(&self, left:&Scoreboard, right:&Scoreboard) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let oper_eq = format!("{}=", self.to_str());
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Arithmetic(self.clone()),
            right.datatype.clone()
        );

        match (left.datatype, right.datatype) {
            // [Left] [+-*/%] [Right]
            (Types::Int, Types::Int) => Ok(
                f_constract
                    .calc_score(left, oper_eq, right)
                    .build()
            ),
            // [Left] [+-*/%] ([Right * MAG] / [MAG])
            (Types::Int, Types::Float) => {
                let adjust_temp = get_type_adjusted_temp(Types::Int);
                Ok(
                    f_constract
                        .assign_score(left, right)
                        .intify(&adjust_temp)
                        .calc_score(left, oper_eq, &adjust_temp)
                        .free(&adjust_temp)
                        .build()
                )
            },
            // [Left * MAG] [+-*/%] [Right * MAG]
            (Types::Float, Types::Int) => {
                let adjust_temp = get_type_adjusted_temp(Types::Int);
                Ok(
                    f_constract
                        .assign_score(left, &adjust_temp)
                        .fltify(&adjust_temp)
                        .calc_score(left, oper_eq, &adjust_temp)
                        .free(&adjust_temp)
                        .build()
                )
            },
            (Types::Float, Types::Float) => match self {
                // Undefined
                Arithmetic::Sur => Err(undefined_operation_occured),
                // ([Left * MAG] * [Right * MAG]) / MAG = ([Left] * [Right]) * MAG
                Arithmetic::Mul => Ok(
                    f_constract
                        .calc_score(left, oper_eq, right)
                        .calc_num(left, "/=".to_string(), FLOAT_MAGNIFICATION)
                        .build()
                ),
                // ([Left * MAG] * MAG) / [Right * MAG] = ([Left] / [Right]) * MAG
                Arithmetic::Div => Ok(
                    f_constract
                        .calc_num(left, "*=".to_string(), FLOAT_MAGNIFICATION)
                        .calc_score(left, oper_eq, right)
                        .build()
                ),
                // [Left * MAG] [+-] [Right * MAG] = ([Left] [+-] [Right]) * MAG
                Arithmetic::Add | Arithmetic::Rem => Ok(
                    f_constract
                        .calc_score(left, oper_eq, right)
                        .build()
                )
            },
            // Undefined
            _ => Err(undefined_operation_occured)
        }
    }
    fn calc_int(&self, left:&Scoreboard, right:i32) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let oper_eq = format!("{}=", self.to_str());
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Arithmetic(self.clone()),
            Types::Int
        );
        let scaled_right = match left.datatype {
            Types::Int => right,
            Types::Float => right * FLOAT_MAGNIFICATION,
            _ => {return Err(undefined_operation_occured);}
        };
        match self {
            Arithmetic::Add => Ok(
                f_constract
                    .add_rem_num(left, "add".to_string(), scaled_right)
                    .build()
            ),
            Arithmetic::Rem => Ok(
                f_constract
                    .add_rem_num(left, "remove".to_string(), scaled_right)
                    .build()
            ),
            _ => Ok(
                f_constract
                    .calc_num(left, oper_eq, right)
                    .build()
            )
        }
    }
    fn calc_float(&self, left:&Scoreboard, right:f32) -> Result<Vec<ScoreAST>, CompileError> {
        let mut f_constract = FormulaConstructer::new();
        let oper_eq = format!("{}=", self.to_str());
        let undefined_operation_occured = CompileError::UndefinedOperation(
            left.datatype.clone(),
            Oper::Arithmetic(self.clone()),
            Types::Int
        );
        match self {
            Arithmetic::Add | Arithmetic::Rem => {
                let add_rem = match self {
                    Arithmetic::Add => "add",
                    Arithmetic::Rem => "remove",
                    _ => unreachable!()
                }.to_string();
                match left.datatype {
                    // [Left * MAG] [+-*/%] ([Right] * [MAG]).floor()
                    Types::Float => Ok(
                        f_constract
                            .add_rem_num(left, add_rem, (right * FLOAT_MAGNIFICATION as f32).floor() as i32)
                            .build()
                    ),
                    // [Left] [+-*/%] [Right].floor()
                    Types::Int => Ok(
                        f_constract
                            .add_rem_num(left, add_rem, right.floor() as i32)
                            .build()
                    ),
                    // Undefined
                    _ => Err(undefined_operation_occured)
                }
            },
            Arithmetic::Sur => Err(undefined_operation_occured),
            Arithmetic::Mul => match left.datatype {
                // Left: int, Right: float
                // ([Left] * MAG) * ([Right] * MAG).floor() / MAG^2 = ([Left] * [Right]).floor()
                Types::Int => Ok({
                    let type_adjusted = get_type_adjusted_temp(Types::Float);
                    f_constract
                        .assign_score(&type_adjusted, left)
                        .fltify(&type_adjusted)
                        .calc_num(&type_adjusted, oper_eq, (right * FLOAT_MAGNIFICATION as f32) as i32)
                        .intify(&type_adjusted)
                        .intify(&type_adjusted)
                        .assign_score(left, &type_adjusted)
                        .free(&type_adjusted)
                        .build()
                }),
                // Left: float, Right: float
                // [Left * MAG] * ([Right] * MAG).floor() / MAG = ([Left] * [Right]) * MAG
                Types::Float => Ok(
                    f_constract
                        .calc_num(left, oper_eq, (right * FLOAT_MAGNIFICATION as f32) as i32)
                        .intify(left)
                        .build()
                ),
                // Undefined
                _ => Err(undefined_operation_occured)
            },
            Arithmetic::Div => match left.datatype {
                // Left: int, Right: float
                // ([Left] * MAG) * [Right * MAG] / MAG = ([Left] * [Right]).floor()
                Types::Int => Ok({
                    let type_adjusted = get_type_adjusted_temp(Types::Float);
                    f_constract
                        .assign_score(&type_adjusted, left)
                        .fltify(&type_adjusted)
                        .fltify(&type_adjusted)
                        .calc_num(&type_adjusted, oper_eq, (right * FLOAT_MAGNIFICATION as f32) as i32)
                        .intify(&type_adjusted)
                        .assign_score(left, &type_adjusted)
                        .free(&type_adjusted)
                        .build()
                }),
                // Left: float, Right: float
                // [Left * MAG] * [Right * MAG] / MAG = ([Left] * [Right]) * MAG
                Types::Float => Ok(
                    f_constract
                        .calc_num(left, oper_eq, (right * FLOAT_MAGNIFICATION as f32) as i32)
                        .intify(left)
                        .build()
                ),
                _ => Err(undefined_operation_occured)
            },
        }
    }
}