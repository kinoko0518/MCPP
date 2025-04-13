use std::{collections::HashMap, vec};

pub mod evaluater;
pub mod tokeniser;

use evaluater::{Scoreboard, Types};

#[derive(Debug, PartialEq)]
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
}