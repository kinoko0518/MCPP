use ast::serialiser::MCFunction;
use ast::serialiser::MCFunctionizable;
use ast::syntax_analyser;
use ast::SyntaxError;
use evaluater::scoreboard::command_ast::CommandAST;
use evaluater::Oper;

pub mod save;
pub mod evaluater;
pub mod tokeniser;
pub mod ast;

use crate::compiler::ast::serialiser::IToken;
use crate::evaluater::Type;
use crate::evaluater::Scoreboard;

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
    fn to_type(&self) -> Option<Type> {
        match self {
            Token::IntType => Some(Type::Int),
            Token::FltType => Some(Type::Float),
            Token::BlnType => Some(Type::Bool),
            _ => None
        }
    }
}
#[derive(Debug)]
pub enum CompileError {
    ASyntaxErrorOccured(SyntaxError),
    InvalidTokenInAFormula(Token),
    EmptyFormulaGiven,
    UndefinedIdentifierReferenced(String),
    UnknownTypeSpecialised(Token),
    LHSDoesntSatisfyValidFormat,
    InvalidRHS(IToken),
    TheTokenIsntValue(IToken),
    InvalidFormulaStructure(String),
    UnsupportedLiteralType(IToken),
    UndefinedOperation(Type, Oper, Type),
    UnbalancedParentheses,
    TheTypeOfAIndentifierWontBeConfirmed(String)
}
impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            CompileError::ASyntaxErrorOccured(a) => a.to_string(),
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
            CompileError::UnbalancedParentheses => String::from("The number of opening and closing parentheses does not match."),
            CompileError::TheTypeOfAIndentifierWontBeConfirmed(t) => format!("The type of an identifer, {} won't be confirmed at the time of compiling.", t)
        };
        write!(f, "{}", result)
    }
}
pub struct Compiler {
    pub namespace: String,
    pub compiled: Vec<MCFunction>,
    pub variables: Vec<Scoreboard>,
    pub functions: Vec<MCFunction>,
    pub scope: Vec<String>
}
impl From<&str> for Compiler {
    fn from(value: &str) -> Self {
        Self {
            namespace: value.to_string(),
            compiled: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
            scope: Vec::new()
        }
    }
}
impl Compiler {
    fn get_score(&self, name:&String) -> Option<&Scoreboard> {
        self
            .variables
            .iter()
            .find(|score| &score.name == name)
    }
    fn get_func(&self, name:&String) -> Option<&MCFunction> {
        self
            .functions
            .iter()
            .find(|func| &func.name == name)
    }
    fn leave_current_scope(&mut self) -> Vec<CommandAST> {
        let mut res:Vec<CommandAST> = Vec::new();
        for v in 0..=self.variables.len() {
            if self.variables.get(v).unwrap().scope.len() >= self.scope.len() {
                res.extend(self.variables.remove(v).free());
            }
        }
        res
    }
    fn serialise_mcfunction<T:MCFunctionizable>(&mut self, mcfunctionizable:&T) -> Result<(), CompileError> {
        let compiled = mcfunctionizable.mcfunctionate(self)?;
        self.compiled.push(compiled.clone());
        self.functions.push(compiled);
        Ok(())
    }
    pub fn evaluate(mut self, target:String) -> Result<String, CompileError> {
        let mut s_analyser = syntax_analyser
            ::SyntaxAnalyser
            ::from(tokeniser::tokenize(target));
        let codeblock = match s_analyser.get_block() {
            Ok(o) => o,
            Err(e) => Err(CompileError::ASyntaxErrorOccured(e))?
        };
        let mcfunc = codeblock.mcfunctionate(&mut self)?;
        Ok(mcfunc.inside.clone())
    }
}