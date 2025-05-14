mod scoreboard;

use super::CompileError;
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

fn get_const(value:&FToken) -> Result<(String, Scoreboard), CompileError> {
    let score = Scoreboard {
        name : value.to_string(),
        scope : vec!["CONSTANT".to_string()],
        datatype : match get_datatype(value) {
            Some(s) => s,
            None => return Err(CompileError::TheTokenIsntValue(value.clone()))
        }
    };
    Ok((score.assign(&value)?, score))
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
fn to_rpn(input:&Vec<FToken>) -> Result<Vec<FToken>, CompileError> {
    let mut queue:Vec<&FToken> = Vec::new();
    let mut stack:Vec<&FToken> = Vec::new();

    if input.is_empty() {
        return Err(CompileError::EmptyFormulaGiven);
    }

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
                    unreachable!()
                }
            }
        }
    }
    while !stack.is_empty() {
        queue.push(&stack.pop().unwrap());
    }
    Ok(queue.iter().map(|t| (**t).clone()).collect::<Vec<FToken>>())
}
fn get_datatype(token:&FToken) -> Option<Types> {
    match token {
        FToken::Int(_) => Some(Types::Int),
        FToken::Flt(_) => Some(Types::Float),
        FToken::Scr(s) => Some(s.datatype),
        _ => None
    }
}

fn eval_rpn(store_to:&Scoreboard, rpn:&Vec<FToken>) -> Result<String, CompileError> {
    let mut commands:Vec<String> = Vec::new();
    let mut stack:Vec<Scoreboard> = Vec::new();
    let mut temps_to_free:Vec<Scoreboard> = Vec::new();

    for token in rpn.iter() {
        if is_value(token) {
            match token {
                FToken::Scr(s) => {
                    stack.push(s.clone());
                },
                FToken::Int(_) | FToken::Flt(_) | FToken::Bln(_) => {
                    let temp_val = Scoreboard {
                        name: format!("EVAL_LIT_{}", scoreboard::generate_random_id(scoreboard::TEMP_ID_LEN)),
                        scope: vec!["TEMP".to_string()],
                        datatype: get_datatype(token).unwrap_or_else(|| {
                            panic!("Literal token has no datatype: {:?}", token)
                        }),
                    };
                    commands.push(temp_val.assign(token)?);
                    stack.push(temp_val.clone());
                    temps_to_free.push(temp_val);
                },
                _ => unreachable!("is_value returned true for non-value token")
            }
        } else if let FToken::Oper(operator) = token {
            let rhs_board = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;
            let lhs_board = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;

            commands.push(lhs_board.calc(operator, &FToken::Scr(rhs_board.clone()))?);
            stack.push(lhs_board.clone());
        } else {
             unreachable!("Non-operator, non-value token found in RPN: {:?}", token);
        }
    }
    commands.push(store_to.assign(&&FToken::Scr(stack.pop().unwrap()))?);
    for tmp in temps_to_free {
        commands.push(tmp.free());
    }
    Ok(commands.join("\n"))
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

pub fn evaluate(compiler:&mut Compiler, formula:&Vec<Token>) -> Result<String, CompileError> {
    let rhs;
    let lhs;

    {
        let mut is_after_eq:bool = false;
        let mut rhs_temp:Vec<&Token> = Vec::new();
        let mut lhs_temp:Vec<&Token> = Vec::new();

        for t in formula {
            if let Token::Asn = t {
                is_after_eq = true
            } else {
                if is_after_eq {
                    rhs_temp.push(t)
                } else {
                    lhs_temp.push(t)
                }
            }
        }

        lhs = if lhs_temp.is_empty() {
            None
        } else {
            Some(lhs_temp)
        };
        rhs = rhs_temp;
    }
    
    let mut f_formula = Vec::new();
    for t in rhs {
        f_formula.push(match to_ftoken(compiler, t) {
            Some(o) => o,
            None => return Err(CompileError::InvalidTokenInAFormula(t.clone()))
        })
    }
    let f_formula = f_formula;
    let rpn = &to_rpn(&f_formula)?;
    let formula_datatype = get_datatype(rpn.get(0).unwrap()).unwrap();
    let eval_temp = Scoreboard {
        name: "EVAL".to_string(),
        scope: compiler.scope.clone(),
        datatype: formula_datatype.clone()
    };
    let store_to:Scoreboard = match lhs {
        Some(s) => {
            if let [Token::Ident(t)] = &s[..] {
                match compiler.get_variable(&t) {
                    Some(s) => s.clone(),
                    None => return Err(CompileError::UndefinedIdentifierReferenced(t.clone()))
                }
            } else if let [Token::Let, Token::Ident(t)] = &s[..] {
                let _result = &compiler.local_variables.insert(
                    t.clone(),
                    Scoreboard {
                        name: t.clone(),
                        scope: compiler.scope.clone(),
                        datatype: formula_datatype
                    }
                );
                compiler.get_variable(&t).unwrap().clone()
            } else if let [Token::Let, Token::Ident(id), Token::Colon, t] = &s[..] {
                Scoreboard {
                    name: id.clone(),
                    scope: compiler.scope.clone(),
                    datatype: match t {
                        Token::IntType => Types::Int,
                        Token::FltType => Types::Float,
                        _ => return Err(CompileError::UnknownTypeSpecialised(t.to_owned().clone()))
                    }
                }
            } else {
                return Err(CompileError::LHSDoesntSatisfyValidFormat);
            }
        },
        None => eval_temp
    };
    compiler.local_variables.insert(store_to.name.clone(), store_to.clone());
    Ok(eval_rpn(&store_to, &rpn)?)
}