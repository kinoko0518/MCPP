pub mod serialiser;
pub mod syntax_analyser;

use super::Type;
use crate::compiler::Oper;
use crate::compiler::Token;

#[derive(Debug, Clone)]
pub enum FToken {
    Int(i32),
    Flt(f32),
    Bln(bool),
    Scr(String),
    Fnc(String, Tuple),
    Mcr(String, Tuple),
    Str(String),
    Oper(Oper),
    LParen,
    RParen
}
impl FToken {
    fn is_value(&self) -> bool {
        matches!(
            self,
            FToken::Int(_) | FToken::Flt(_) |
            FToken::Bln(_) | FToken::Scr(_) |
            FToken::Fnc(_, _) | FToken::Mcr(_, _)
        )
    }
    fn is_operator(&self) -> bool {
        matches!(self, FToken::Oper(_))
    }
}
impl std::fmt::Display for FToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            FToken::Int(i) => i.to_string(),
            FToken::Flt(f) => f.to_string(),
            FToken::Bln(b) => b.to_string(),
            FToken::Scr(s) => s.clone(),
            FToken::Str(s) => s.clone(),
            FToken::Oper(o) => o.to_str().to_string(),
            FToken::LParen => "(".to_string(),
            FToken::RParen => ")".to_string(),
            FToken::Fnc(n, _) => format!("{}(...)", n),
            FToken::Mcr(m, _) => format!("{}!(...)", m),
        };
        write!(f, "{}", res)
    }
}

#[derive(Debug, Clone)]
pub enum SyntaxError {
    ExpectedAToken(String),
    EmptyFormula,
    InvalidFormAs(String),
    UndefinedOperationFound(Type, Oper, Type),
    NotEnoughOperand,
    OperatorAtInvalidPosition(Oper),
    UnbalancedBraces,
    UnbalancedParentheses,
    TokenEndsUnexpectedly,
    ALineMustntStartWith(Token),
    InvalidTokenInAFormula(Token),
    ArgumentCountMismatch,
}
impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::ExpectedAToken(t) => format!("A token, {} is expected.", t),
            Self::EmptyFormula => format!("Empty formula was given."),
            Self::InvalidFormAs(t) => format!("The given tokens have an invalid form as {}", t),
            Self::UndefinedOperationFound(l, o, r) => format!("An undefined operation, {} {} {} occured.", l, o, r),
            Self::NotEnoughOperand => format!("Not enough operand. Add a operand or remove the last operator."),
            Self::OperatorAtInvalidPosition(o) => format!("An operator, {} is in invalid position.", o),
            Self::UnbalancedBraces => format!("Unbalanced braces found. Please make sure that it's closed."),
            Self::UnbalancedParentheses => format!("Unbalanced parentheses found. Please make sure that it's closed."),
            Self::TokenEndsUnexpectedly => format!("Token was ended unexpectedly. This syntax perhaps expects more tokens."),
            Self::ALineMustntStartWith(t) => format!("A line mustn't starts with a token, {:?}.", t),
            Self::InvalidTokenInAFormula(t) => format!("The token, {:?} doesn't constract formulas. It mustn't be in a formula.", t),
            Self::ArgumentCountMismatch => format!("The function was given fewer or too many arguments.")
        })
    }
}
#[derive(Debug, Clone)]
enum AST {
    Formula(Formula),
    CodeBlock(CodeBlock),
    IfSyntax(IfSyntax),
    WhileSyntax(WhileSyntax),
    LetStatement(VariableDefinement),
    Assignment(Assignment)
}
#[derive(Debug, Clone)]
struct Formula {
    formula_tokens: Vec<FToken>
}
impl From<Vec<FToken>> for Formula {
    fn from(value: Vec<FToken>) -> Self {
        Formula { formula_tokens: value }
    }
}
impl Formula {
    fn to_rpn(self) -> Result<RPNFormula, SyntaxError> {
        let mut queue:Vec<&FToken> = Vec::new();
        let mut stack:Vec<&FToken> = Vec::new();

        if self.formula_tokens.is_empty() {
            return Err(SyntaxError::EmptyFormula);
        }
        for current in &self.formula_tokens {
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
                            queue.push(stack.pop().unwrap());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                stack.push(current);
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
                                _ => queue.push(top),
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
            queue.push(stack.pop().unwrap());
        }
        Ok(RPNFormula {
            formula_tokens: queue
                .iter()
                .map(|t| t.clone().clone())
                .collect::<Vec<FToken>>()
        })
    }
}

#[derive(Debug, Clone)]
pub struct RPNFormula {
    formula_tokens: Vec<FToken>
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    inside: Vec<AST>
}

#[derive(Debug, Clone)]
pub struct IfSyntax {
    condition: RPNFormula,
    block: CodeBlock
}

#[derive(Debug, Clone)]
pub struct WhileSyntax {
    condition: RPNFormula,
    block: CodeBlock
}

#[derive(Debug, Clone)]
pub struct VariableDefinement {
    identifier: String,
    datatype: Option<Type>,
    initialise: Option<Assignment>
}

#[derive(Debug, Clone)]
pub struct Assignment {
    lhs: String,
    rhs: RPNFormula
}

#[derive(Debug, Clone)]
pub struct Arguments {
    name: String,
    datatype: Type
}

#[derive(Debug, Clone)]
pub struct FunctionDefinement {
    func_name: String,
    datatype: Type,
    arguments: Vec<Arguments>,
    block: CodeBlock
}

#[derive(Debug, Clone)]
pub struct Tuple {
    inside : Vec<Formula>
}