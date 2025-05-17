use std::{collections::HashMap, vec};

use evaluater::Oper;
use evaluater::scoreboard::arithmetic_operation::Arithmetic;
use evaluater::scoreboard::logical_operation::Logical;
use evaluater::scoreboard::comparison_operation::Comparison;
use crate::compiler::evaluater::scoreboard::command_ast::Serialise;

pub mod evaluater;
pub mod tokeniser;

use evaluater::{evaluate, FToken, Scoreboard, Types};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Identifier and literal
    Ident(String), // variable / function name
    Int(i32),      // num+
    Flt(f32),      // num*.num+
    Bln(bool),     // true / false
    Str(String),   // "..."
    MCId(String),  // $...:... or $(minecraft:)...

    // Operator
    Asn, // =
    Add, // +
    Rem, // - 
    Mul, // *
    Div, // /
    Sur, // %
    
    // Compare
    Eq,  // ==
    NEq, // !=
    LEt, // <=
    REt, // >=
    Lt,  // <
    Gt,  // >

    // Logical Operator
    Neg, // !
    And, // &
    Or,  // |

    // Arrows
    Arr, // ->
    FArr,// =>

    // Delimiters
    Comma,     // ,
    Dot,       // .
    Semicolon, // ;
    Colon,     // :
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]

    // Keywords
    Let, // Values binding
    Fn, If, Else, While, For, // Sentense specifiers
    IntType, FltType, BlnType, NoneType, // Types. Float containt how many decimal places does it ensures.
    Return, // Returning a value
}
impl Token {
    fn to_ftoken(&self, compiler:&Compiler) -> Option<FToken> {
        match self {
            // Arithmetic operations
            Token::Add => Some(FToken::Oper(Oper::Arithmetic(Arithmetic::Add))),
            Token::Rem => Some(FToken::Oper(Oper::Arithmetic(Arithmetic::Rem))),
            Token::Mul => Some(FToken::Oper(Oper::Arithmetic(Arithmetic::Mul))),
            Token::Div => Some(FToken::Oper(Oper::Arithmetic(Arithmetic::Div))),
            Token::Sur => Some(FToken::Oper(Oper::Arithmetic(Arithmetic::Sur))),
            
            // Logical operations
            Token::And => Some(FToken::Oper(Oper::Logical(Logical::And))),
            Token::Or  => Some(FToken::Oper(Oper::Logical(Logical::Or))),
            Token::Neg => Some(FToken::Oper(Oper::Logical(Logical::Not))),

            // Comparisons
            Token::Gt  => Some(FToken::Oper(Oper::Comparison(Comparison::Gt))),
            Token::Lt  => Some(FToken::Oper(Oper::Comparison(Comparison::Lt))),
            Token::LEt => Some(FToken::Oper(Oper::Comparison(Comparison::Le))),
            Token::REt => Some(FToken::Oper(Oper::Comparison(Comparison::Ge))),
            Token::Eq  => Some(FToken::Oper(Oper::Comparison(Comparison::Eq))),
            Token::NEq => Some(FToken::Oper(Oper::Comparison(Comparison::Neq))),

            // Parentheses
            Token::LParen => Some(FToken::LParen),
            Token::RParen => Some(FToken::RParen),

            // Literals
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
}
#[derive(Debug)]
pub enum CompileError {
    InvalidTokenInAFormula(Token),
    EmptyFormulaGiven,
    UndefinedIdentifierReferenced(String),
    UnknownTypeSpecialised(Token),
    LHSDoesntSatisfyValidFormat,
    InvalidRHS(evaluater::FToken),
    TheTokenIsntValue(evaluater::FToken),
    InvalidFormulaStructure(String),
    UnsupportedLiteralType(evaluater::FToken),
    UndefinedOperation(Types, Oper, Types),
    UnbalancedParentheses
}
impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            CompileError::InvalidTokenInAFormula(t) => format!("An invalid token, {:?} exists in the formula.", t),
            CompileError::EmptyFormulaGiven => String::from("An empty formula was given."),
            CompileError::UndefinedIdentifierReferenced(id) => format!("A identifer, {} was referenced but undefined.", id),
            CompileError::UnknownTypeSpecialised(t) => format!("A token, {:?} isn't valid as type specifier.", t),
            CompileError::LHSDoesntSatisfyValidFormat => String::from("The left hand side doesn't satisfy the valid format."),
            CompileError::InvalidRHS(t) => format!("The rhs, {} can't be assined onto the lhs.", t),
            CompileError::TheTokenIsntValue(t) => format!("The token, {:?} isn't value.", t),
            CompileError::InvalidFormulaStructure(s) => s.clone(),
            CompileError::UnsupportedLiteralType(t) => format!("The token, {} isn't supported as a literal type.", t),
            CompileError::UndefinedOperation(l, o, h) => format!("An unsupported calcation occured, {} {} {}", l, o, h),
            CompileError::UnbalancedParentheses => String::from("The number of opening and closing parentheses does not match.")
        };
        write!(f, "{}", result)
    }
}

pub struct Compiler {
    pub scope: Vec<String>,
    pub inherited_variables: HashMap<String, Scoreboard>,
    pub local_variables: HashMap<String, Scoreboard>,
}
impl Compiler {
    fn get_variable(&self, name:&str) -> Option<&Scoreboard> {
        match self.local_variables.get(name) {
            Some(s) => Some(s),
            None => self.inherited_variables.get(name)
        }
    }
    pub fn new() -> Compiler {
        Compiler {
            scope: Vec::new(),
            inherited_variables: HashMap::new(),
            local_variables: HashMap::new()
        }
    }
    pub fn mark_as_exists(&mut self, name:&str, data_type:&Types) {
        self.local_variables.insert(
            name.to_string(),
            Scoreboard {
                name: name.to_string(),
                scope: vec![],
                datatype: *data_type
            }
        );
    }
    pub fn compile(&mut self, input:&str) -> Result<String, CompileError> {
        let lines = {
            let mut lines:Vec<Vec<Token>> = Vec::new();
            let mut current_line:Vec<Token> = Vec::new();
            for t in tokeniser::tokenize(input.to_string()) {
                if let Token::Semicolon = t {
                    lines.push(current_line.clone());
                    current_line.clear();
                } else {
                    current_line.push(t.clone());
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line.clone());
            }
            lines
        };
        let result = {
            let mut result = Vec::new();
            for l in lines {
                result.extend(evaluate(self, &l)?)
            }
            result
                .iter()
                .map(|ast| ast.clone().serialise()).collect::<Vec<String>>()
        }.join("\n");
        Ok(result)
    }
}