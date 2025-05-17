pub mod scoreboard;

use super::CompileError;
use super::Token;
use super::Compiler;

use scoreboard::command_ast::ScoreAST;
pub use scoreboard::Scoreboard;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Types { Int, Float, Bool }
impl std::fmt::Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Types::Bool => "Bool",
            Types::Float => "Float",
            Types::Int => "Int"
        })
    }
}
trait Operator {
    fn get_priority(&self) -> u32;
    fn to_str(&self) -> &str;
    fn calc(&self, left:&Scoreboard, right:&FToken) -> Result<Vec<ScoreAST>, CompileError>;
}
#[derive(Debug, Clone)]
pub enum Oper {
    Arithmetic(scoreboard::arithmetic_operation::Arithmetic),
    Logical(scoreboard::logical_operation::Logical),
    Comparison(scoreboard::comparison_operation::Comparison)
}
impl std::fmt::Display for Oper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Arithmetic(a) => a.to_str(),
            Self::Comparison(c) => c.to_str(),
            Self::Logical(l) => l.to_str()
        })
    }
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
        matches!(self, FToken::Int(_) | FToken::Flt(_) | FToken::Bln(_) | FToken::Scr(_))
    }
    fn is_operator(&self) -> bool {
        matches!(self, FToken::Oper(_))
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
fn eval_rpn(store_to: &Scoreboard, rpn: &Vec<FToken>) -> Result<Vec<ScoreAST>, CompileError> {
    let mut commands: Vec<ScoreAST> = Vec::new();
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
                name: format!("CALC_RESULT_{}", scoreboard::generate_random_id(scoreboard::TEMP_ID_LEN)),
                scope: vec!["TEMP".to_string()],
                datatype: lhs_board.get_datatype_or().unwrap()
            };
            // let TEMP.CALC_RESULT_XXX = LHS;
            // TEMP.CALC_RESULT_XXX [OPERATOR]= RHS;
            commands.extend(result_container.assign(&lhs_board)?);
            commands.extend(
                match operator {
                    Oper::Arithmetic(a) => a.calc(&result_container, &rhs_board)?,
                    Oper::Comparison(c) => c.calc(&result_container, &rhs_board)?,
                    Oper::Logical(l) => l.calc(&result_container, &rhs_board)?
                }
            );
            // Add the scoreboard to temp boards to free the score after it become unnecessary
            temp_scores.push(result_container.clone());
            stack.push(FToken::Scr(result_container));
        } else {
            // There's no token that isn't value nor operator in formula token, right?
            unreachable!("Non-operator, non-value token found in RPN: {:?}", token);
        }
    }
    if stack.len() == 1 {
        commands.extend(store_to.assign(&&stack.pop().unwrap())?);
    } else {
        return Err(CompileError::UnbalancedParentheses);
    }
    // Free temp scores generated for calcation
    for tmp in temp_scores {
        commands.extend(tmp.free());
    }
    Ok(commands)
}
// Evaluate a formula constructed with middle notated lhs and rhs
pub fn evaluate(compiler:&mut Compiler, formula:&Vec<Token>) -> Result<Vec<ScoreAST>, CompileError> {
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
    
    // FTokenising formula
    let mut f_formula = Vec::new();
    for t in rhs {
        f_formula.push(match t.to_ftoken(compiler) {
            Some(o) => o,
            None => return Err(CompileError::InvalidTokenInAFormula(t.clone()))
        })
    }
    // Make it immutable
    let f_formula = f_formula;
    let rpn = &to_rpn(&f_formula)?;
    let formula_datatype = rpn.get(0).unwrap().get_datatype_or().unwrap();
    // The scoreboard will be used as temporary result container when there's no lhs.
    let eval_temp = Scoreboard {
        name: "EVAL_RESULT".to_string(),
        scope: compiler.scope.clone(),
        datatype: formula_datatype.clone()
    };
    // Specifing the scoreboard where the result will be stored after the calculation.
    let store_to:Scoreboard = match lhs {
        Some(s) => {
            // [Identifier] = [Formula]
            if let [Token::Ident(t)] = &s[..] {
                match compiler.get_variable(&t) {
                    Some(s) => s.clone(),
                    None => return Err(CompileError::UndefinedIdentifierReferenced(t.clone()))
                }
            // let [Identifier] = [Formula]
            } else if let [Token::Let, Token::Ident(id)] = &s[..] {
                let _result = &compiler.local_variables.insert(
                    id.clone(),
                    Scoreboard {
                        name: id.clone(),
                        scope: compiler.scope.clone(),
                        datatype: formula_datatype
                    }
                );
                compiler.get_variable(&id).unwrap().clone()
            // let [Identifier]:[Type] = [Formula]
            } else if let [Token::Let, Token::Ident(id), Token::Colon, t] = &s[..] {
                let _result = &compiler.local_variables.insert(
                    id.clone(),
                    Scoreboard {
                        name: id.clone(),
                        scope: compiler.scope.clone(),
                        datatype: match t {
                            Token::IntType => Types::Int,
                            Token::FltType => Types::Float,
                            _ => return Err(CompileError::UnknownTypeSpecialised(t.to_owned().clone()))
                        }
                    }
                );
                compiler.get_variable(&id).unwrap().clone()
            // Other patterns excluding [Formula], it must generate an error
            } else {
                return Err(CompileError::LHSDoesntSatisfyValidFormat);
            }
        },
        // [Formula]
        None => eval_temp
    };
    Ok(eval_rpn(&store_to, &rpn)?)
}