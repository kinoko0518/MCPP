use super::Operator;
use super::Types;
use super::FToken;

const NAMESPACE:&str = "MCPP.var";
const FLOAT_MAGNIFICATION:i32 = 1000;
#[derive(Debug, Clone)]
pub struct Scoreboard {
    pub name : String,
    pub scope : Vec<String>,
    pub datatype : Types
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

impl Scoreboard {
    fn get_calc_temp() -> Scoreboard {
        Scoreboard {
            name: "LITERAL".to_string(),
            scope: vec!["TEMP".to_string()],
            datatype: Types::Int
        }
    }
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
            _ => panic!("Invalid operation!")
        }
    }
    
    pub fn calc_score(&self, operator:&Operator, right:&Scoreboard) -> String {
        match self.datatype {
            Types::Int => match right.datatype {
                Types::Int => self.pure_calc_score(operator, right),
                Types::Float => {
                    let temp = Self::get_calc_temp();
                    format!(
                        "{}\n{}\n{}",
                        temp.pure_assign_score(right),
                        temp.pure_calc_num(&Operator::Div, FLOAT_MAGNIFICATION),
                        self.pure_calc_score(operator, &temp)
                    )
                }
            },
            Types::Float => match right.datatype {
                Types::Int => {
                    let temp =  Self::get_calc_temp();
                    format!(
                        "{}\n{}\n{}",
                        temp.pure_assign_score(right),
                        temp.pure_calc_num(&Operator::Mul, FLOAT_MAGNIFICATION),
                        self.pure_calc_score(operator, &temp)
                    )
                },
                Types::Float => match operator {
                    Operator::Add | Operator::Rem => self.pure_calc_score(operator, right),
                    Operator::Mul => format!(
                        "{}\n{}",
                        self.pure_calc_score(operator, right),
                        self.pure_calc_num(&Operator::Div, FLOAT_MAGNIFICATION)
                    ),
                    Operator::Div => format!(
                        "{}\n{}",
                        self.pure_calc_num(&Operator::Mul, FLOAT_MAGNIFICATION),
                        self.pure_calc_score(operator, right),
                    ),
                    Operator::Sur => todo!(),
                    Operator::Asn => self.assign(&FToken::Scr(right.clone()))
                }
            }
        }
    }
    
    pub fn calc(&self, operator:&Operator, right:&FToken) -> String {
        match right {
            FToken::Int(i) => {
                match self.datatype {
                    Types::Int => self.pure_calc_num(operator, *i),
                    Types::Float => self.pure_calc_num(operator, i * FLOAT_MAGNIFICATION)
                }
            },
            FToken::Flt(f) => {
                match self.datatype {
                    Types::Int => self.pure_calc_num(operator, f.round() as i32),
                    Types::Float => self.pure_calc_num(operator, (*f * FLOAT_MAGNIFICATION as f32).round() as i32)
                }
            },
            FToken::Scr(s) => self.calc_score(operator, s),
            _ => panic!("Invalid right hand side.")
        }
    }
    pub fn assign(&self, right:&FToken) -> String {
        match right {
            FToken::Int(i) => match self.datatype {
                Types::Int => self.pure_assign_num(*i),
                Types::Float => self.pure_assign_num(i * FLOAT_MAGNIFICATION)
            },
            FToken::Flt(f) => match self.datatype {
                Types::Int => self.pure_assign_num(*f as i32),
                Types::Float => self.pure_assign_num((*f * (FLOAT_MAGNIFICATION as f32)) as i32)
            },
            FToken::Scr(s) => {
                match self.datatype {
                    Types::Int => match s.datatype {
                        Types::Int => self.pure_assign_score(s),
                        Types::Float => format!(
                            "{}\n{}",
                            self.pure_assign_score(s),
                            self.pure_calc_num(&Operator::Div, FLOAT_MAGNIFICATION)
                        )
                    },
                    Types::Float => match s.datatype {
                        Types::Int => format!(
                            "{}\n{}",
                            self.pure_assign_score(s),
                            self.pure_calc_num(&Operator::Mul, FLOAT_MAGNIFICATION)
                        ),
                        Types::Float => self.pure_assign_score(s)
                    }
                }
            },
            _ => panic!("Invalid token. {} = {:?} <- Invalid", self.get_mcname(), right)
        }
    }
    pub fn free(&self) -> String {
        format!("scoreboard players reset {} {}", self.get_mcname(), NAMESPACE)
    }
}