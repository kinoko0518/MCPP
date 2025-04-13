mod scoreboard;

use std::vec;

use super::Token;
use super::Compiler;

pub use scoreboard::Scoreboard;

#[derive(Debug, Clone, Copy)]
pub enum Types { Int, Float }
#[derive(Debug, Clone)]
pub enum Operator { Add, Rem, Mul, Div, Sur, Asn }
#[derive(Debug, Clone)]
pub enum FToken {
    Int(i32),
    Flt(f32),
    Bln(bool),
    Scr(Scoreboard),
    Oper(Operator),
    LParen,
    RParen
}
impl std::fmt::Display for FToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            FToken::Int(i) => i.to_string(),
            FToken::Flt(f) => f.to_string(),
            FToken::Bln(b) => b.to_string(),
            FToken::Scr(s) => s.get_mcname(),
            FToken::Oper(o) => scoreboard::oper_to_str(o).to_string(),
            FToken::LParen => "(".to_string(),
            FToken::RParen => ")".to_string()
        };
        write!(f, "{}", res)
    }
}
impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", scoreboard::oper_to_str(self))
    }
}

fn get_const(value:&FToken) -> Option<(String, Scoreboard)> {
    let score = Scoreboard {
        name : value.to_string(),
        scope : vec!["CONSTANT".to_string()],
        datatype : get_datatype(value)?
    };
    Some((score.assign(&value), score))
}
fn get_priority(given:&FToken) -> Option<u32> {
    match given {
        FToken::Oper(o) => match o {
            Operator::Mul | Operator::Div | Operator::Sur => Some(3),
            Operator::Add | Operator::Rem => Some(2),
            Operator::Asn => Some(1),
        },
        _ => None
    }
}
fn is_value(given:&FToken) -> bool {
    match given {
        FToken::Int(_) | FToken::Flt(_) | FToken::Bln(_) | FToken::Scr(_) => true,
        _ => false
    }
}
fn is_operator(given:&FToken) -> bool {
    match given {
        FToken::Oper(_) => true,
        _ => false
    }
}
fn to_rpn(input:&Vec<FToken>) -> Vec<&FToken> {
    let mut queue:Vec<&FToken> = Vec::new();
    let mut stack:Vec<&FToken> = Vec::new();

    for current in input.iter() {
        if is_value(current) {
            queue.push(current);
            continue;
        } else if is_operator(current) {
            while let Some(top) = stack.last() {
                if !matches!(top, FToken::Oper(_)) {
                    break;
                }
                if get_priority(current).unwrap() <= get_priority(&top).unwrap() {
                    queue.push(&stack.pop().unwrap());
                } else {
                    break;
                }
            }
            stack.push(&current);
            continue;
        } else {
            match current {
                FToken::LParen => {
                    stack.push(current);
                },
                FToken::RParen => {
                    loop {
                        let top = stack.pop().unwrap();
                        match top {
                            FToken::LParen => break,
                            _ => queue.push(&top),
                        }
                    }
                },
                _ => {
                    panic!("Invalid formula. The token, {:?} is not value nor operator.", current);
                }
            }
        }
    }
    while !stack.is_empty() {
        queue.push(&stack.pop().unwrap());
    }
    queue
}
fn get_datatype(token:&FToken) -> Option<Types> {
    match token {
        FToken::Int(_) => Some(Types::Int),
        FToken::Flt(_) => Some(Types::Float),
        FToken::Scr(s) => Some(s.datatype),
        _ => None
    }
}

fn eval_rpn(store_to:&Scoreboard, rpn:&Vec<&FToken>) -> Vec<String> {
    let mut res = Vec::new();
    let mut queue = Vec::new();
    let mut rpn = rpn.clone();
    
    rpn.reverse();

    let top_element = rpn.pop().unwrap();
    res.push(store_to.assign(top_element));
    queue.push(FToken::Scr(store_to.clone()));

    let literal = Scoreboard {
        name: "LITERAL".to_string(),
        scope: vec!["TEMP".to_string()],
        datatype: get_datatype(&top_element).unwrap()
    };

    while !rpn.is_empty() {
        let top = rpn.pop().unwrap();

        if is_operator(top) {
            let operator = match top {
                FToken::Oper(o) => o,
                _ => panic!()
            };
            let rhs = queue.pop().unwrap();
            let lhs = queue.pop().unwrap();

            match lhs {
                FToken::Scr(s) => {
                    res.push(s.calc(operator, &rhs));
                    queue.push(FToken::Scr(s.clone()));
                },
                _ => {
                    res.push(literal.assign(&lhs));
                    res.push(literal.calc(operator, &rhs));
                    res.push(store_to.assign(&FToken::Scr(literal.clone())));
                    queue.push(FToken::Scr(store_to.clone()));
                }
            };
        } else {
            queue.push(top.clone());
        }
    }

    res
}
fn to_ftoken(compiler:&Compiler, token:&Token) -> Option<FToken> {
    match token {
        Token::Add => Some(FToken::Oper(Operator::Add)),
        Token::Rem => Some(FToken::Oper(Operator::Rem)),
        Token::Mul => Some(FToken::Oper(Operator::Mul)),
        Token::Div => Some(FToken::Oper(Operator::Div)),
        Token::Sur => Some(FToken::Oper(Operator::Sur)),
        Token::Asn => Some(FToken::Oper(Operator::Asn)),

        Token::LParen => Some(FToken::LParen),
        Token::RParen => Some(FToken::RParen),
        
        Token::Int(i) => Some(FToken::Int(*i)),
        Token::Flt(f) => Some(FToken::Flt(*f)),
        Token::Bln(b) => Some(FToken::Bln(*b)),
        Token::Ident(id) => match compiler.get_variable(id) {
            Some(s) => Some(FToken::Scr(s.clone())),
            None => None
        },
        
        _ => None
    }
}

pub fn evaluate(compiler:&mut Compiler, formula:&Vec<Token>) -> Vec<String> {
    let to_f_formula = |f: &Vec<&Token>| {
        f
            .iter()
            .map(|t| to_ftoken(compiler, t).expect(format!("Invalid token in formula. The token -> {:?}", t).as_str()))
            .collect::<Vec<FToken>>()
    };
    let formula_datatype = |from:&Vec<&Token>| {
        get_datatype(
            to_rpn(&to_f_formula(from))
                .iter()
                .filter(|t| is_value(t))
                .collect::<Vec<&&FToken>>()
                .get(0)
                .unwrap()
        ).unwrap()
    };

    let mut rhs:Vec<&Token> = Vec::new();
    let lhs = if let (Some(Token::Ident(i)), Some(Token::Asn)) = (formula.get(0), formula.get(1)) {
        // Existing pattern
        // Example: a = 100 (A variable, a is predefined)
        rhs = formula[2..].iter().collect::<Vec<&Token>>();
        Some(compiler.get_variable(i).expect("Referencing variable is undefined.").clone())
    } else if let (
        Some(Token::Let),
        Some(Token::Ident(i)),
        Some(Token::Asn)
    ) = (formula.get(0), formula.get(1), formula.get(2)) {
        // Untyped pattern
        // Example: let a = 100
        rhs = formula[3..].iter().collect::<Vec<&Token>>();
        Some(Scoreboard {
            name: i.clone(),
            scope: compiler.scope.clone(),
            datatype: formula_datatype(&rhs)
        })
    } else if let (
        Some(Token::Let),
        Some(Token::Ident(i)),
        Some(Token::Colon),
        Some(datatype),
        Some(Token::Asn)
    ) = (formula.get(0), formula.get(1), formula.get(2), formula.get(3), formula.get(4)) {
        // Typed pattern
        // Example: let a:int = 100
        rhs = formula[5..].iter().collect::<Vec<&Token>>();
        Some(
            Scoreboard {
                name: i.clone(),
                scope: compiler.scope.clone(),
                datatype: match datatype {
                    Token::IntType => Types::Int,
                    Token::FltType => Types::Float,
                    _ => panic!("Invalid datatype!")
                }
            }
        )
    } else {
        rhs = formula.iter().collect::<Vec<&Token>>();
        None
    };

    let store_to = match lhs {
        Some(s) => s,
        None => Scoreboard {
            name: "EVAL".to_string(),
            scope: vec!["TEMP".to_string()],
            datatype: formula_datatype(&rhs)
        }
    };

    let f_formula = to_f_formula(&rhs);
    let rpn = &to_rpn(&f_formula);
    eval_rpn(&store_to, rpn)
}