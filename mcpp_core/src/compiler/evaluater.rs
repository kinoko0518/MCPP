mod scoreboard;
pub mod arithmetic_operation;
pub mod comparison_operation;
pub mod logical_operation;

use rand::Rng;
use super::CompileError;
use super::Token;
use super::Compiler;

use arithmetic_operation::Arithmetic;
use logical_operation::Logical;
use comparison_operation::Comparison;

pub use scoreboard::Scoreboard;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Types { Int, Float, Bool }
impl std::fmt::Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Types::Bool => "Bool".to_string(),
            Types::Float => "Float".to_string(),
            Types::Int => "Int".to_string()
        })
    }
}
trait Operator {
    fn get_priority(&self) -> u32;
    fn to_str(&self) -> &str;
}
#[derive(Debug, Clone)]
pub enum Oper {
    Arithmetic(Arithmetic),
    Logical(Logical),
    Comparison(Comparison)
}

#[derive(Debug, Clone)]
pub enum FToken {
    Int(i32),
    Flt(f32),
    Bln(bool),
    Scr(Scoreboard),
    Oper(Oper),
    LParen,
    RParen
}
impl Oper {
    fn get_priority(&self) -> u32 {
        match self {
            Oper::Arithmetic(o) => o.get_priority(),
            Oper::Comparison(o) => o.get_priority(),
            Oper::Logical(o) => o.get_priority(),
        }
    }
    fn to_str(&self) -> &str {
        match self {
            Oper::Arithmetic(o) => o.to_str(),
            Oper::Comparison(o) => o.to_str(),
            Oper::Logical(o) => o.to_str(),
        }
    }
}
impl FToken {
    fn is_value(&self) -> bool {
        match self {
            FToken::Int(_) | FToken::Flt(_) | FToken::Bln(_) | FToken::Scr(_) => true,
            _ => false
        }
    }
    fn is_operator(&self) -> bool {
        if let FToken::Oper(_) = self {
            true
        } else {
            false
        }
    }
    fn get_datatype_or(&self) -> Option<Types> {
        match self {
            FToken::Bln(_) => Some(Types::Bool),
            FToken::Int(_) => Some(Types::Int),
            FToken::Flt(_) => Some(Types::Float),
            FToken::Scr(s) => Some(s.datatype),
            _ => None
        }
    }
}
impl std::fmt::Display for FToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            FToken::Int(i) => i.to_string(),
            FToken::Flt(f) => f.to_string(),
            FToken::Bln(b) => b.to_string(),
            FToken::Scr(s) => s.get_mcname(),
            FToken::Oper(o) => o.to_str().to_string(),
            FToken::LParen => "(".to_string(),
            FToken::RParen => ")".to_string()
        };
        write!(f, "{}", res)
    }
}

pub fn generate_random_id(length:u32) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.random_range('a'..='z') as char)
        .collect::<String>()
}
pub fn get_temp_score(datatype:Types) -> Scoreboard {
    Scoreboard {
        name: format!("CALC_TEMP_{}", generate_random_id(16)),
        scope: vec!["TEMP".to_string()],
        datatype: datatype
    }
}

fn to_rpn(input:&Vec<FToken>) -> Result<Vec<FToken>, CompileError> {
    let mut queue:Vec<&FToken> = Vec::new();
    let mut stack:Vec<&FToken> = Vec::new();

    if input.is_empty() {
        return Err(CompileError::EmptyFormulaGiven);
    }

    for current in input.iter() {
        if current.is_value() {
            queue.push(current);
            continue;
        } else if current.is_operator() {
            while let Some(top) = stack.last() {
                if !matches!(top, FToken::Oper(_)) {
                    break;
                }
                if let (FToken::Oper(l), FToken::Oper(r)) = (current, top) {
                    if l.get_priority() < r.get_priority() {
                        queue.push(&stack.pop().unwrap());
                    } else {
                        break;
                    }
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
/// Evaluate rpn then store result to the "store_to" argument.
fn eval_rpn(store_to: &Scoreboard, rpn: &Vec<FToken>) -> Result<String, CompileError> {
    let mut commands: Vec<String> = Vec::new();
    let mut stack: Vec<FToken> = Vec::new();
    let mut temp_scores:Vec<Scoreboard> = Vec::new();

    for token in rpn.iter() {
        // Move values to the stack
        if token.is_value() {
            match token {
                FToken::Scr(_) | FToken::Int(_)| FToken::Flt(_) | FToken::Bln(_) => {
                    stack.push(token.clone());
                },
                _ => return Err(CompileError::UnsupportedLiteralType(token.clone())),
            }
        // Calcate if a operator poped
        } else if let FToken::Oper(operator) = token {
            // Get lhs and rhs
            let rhs_board = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;
            let lhs_board = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;
            // A calc result expect a container scoreboard
            let result_container = Scoreboard {
                name: format!("CALC_RESULT_{}", generate_random_id(scoreboard::TEMP_ID_LEN)),
                scope: vec!["TEMP".to_string()],
                datatype: lhs_board.get_datatype_or().unwrap()
            };
            // let TEMP.CALC_RESULT_XXX = LHS;
            // TEMP.CALC_RESULT_XXX [OPERATOR]= RHS;
            commands.push(result_container.assign(&lhs_board)?);
            commands.push(match operator {
                Oper::Arithmetic(a) => a.calc(&result_container, &rhs_board)?,
                Oper::Comparison(c) => c.compare_to_get_boolean(&result_container, &rhs_board)?,
                Oper::Logical(l) => l.logicalc(&result_container, &rhs_board)?
            });
            // Add the scoreboard to temp boards to free the score after it become unnecessary
            temp_scores.push(result_container.clone());
            stack.push(FToken::Scr(result_container));
        } else {
            // There's no token that isn't value nor operator in formula token, right?
            unreachable!("Non-operator, non-value token found in RPN: {:?}", token);
        }
    }
    if stack.len() == 1 {
        commands.push(store_to.assign(&&stack.pop().unwrap())?);
    } else {
        return Err(CompileError::UnbalancedParentheses);
    }
    // Free temp scores generated for calcation
    for tmp in temp_scores {
        commands.push(tmp.free());
    }
    Ok(commands.join("\n"))
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
        f_formula.push(match t.to_ftoken(compiler) {
            Some(o) => o,
            None => return Err(CompileError::InvalidTokenInAFormula(t.clone()))
        })
    }
    let f_formula = f_formula;
    let rpn = &to_rpn(&f_formula)?;
    let formula_datatype = rpn.get(0).unwrap().get_datatype_or().unwrap();
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