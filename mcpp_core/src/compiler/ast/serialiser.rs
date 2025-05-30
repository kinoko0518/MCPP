mod macros;

use core::fmt;
use std::vec;

use crate::compiler::Compiler;
use crate::evaluater::scoreboard::command_ast::{ExecuteConstructer, FormulaConstructer, Serialise};
use crate::evaluater::scoreboard::comparison_operation::Comparison;
use crate::evaluater::scoreboard::generate_random_id;
use crate::{compiler::CompileError, evaluater::Scoreboard};

use super::*;
use crate::evaluater::scoreboard::{self, command_ast::CommandAST};
use crate::compiler::evaluater::Operator;

/// Interpreted Token
#[derive(Debug, Clone)]
pub enum IToken {
    Int(i32),
    Flt(f32),
    Bln(bool),
    Scr(Scoreboard),
    Str(String),
    Fnc(MCFunction, Tuple),
    Mcr(String, Tuple),
    Oper(Oper),
    LParen,
    RParen
}
impl IToken {
    fn is_value(&self) -> bool {
        matches!(self, IToken::Scr(_) | IToken::Fnc(_, _) | IToken::Mcr(_, _) | IToken::Int(_) | IToken::Flt(_) | IToken::Bln(_))
    }
    fn get_datatype(&self) -> Option<Type> {
        match self {
            Self::Bln(_) => Some(Type::Bool),
            Self::Flt(_) => Some(Type::Float),
            Self::Fnc(f, _) => Some(f.returning_type.clone()),
            Self::Int(_) => Some(Type::Int),
            Self::Scr(s) => Some(s.datatype.clone()),
            Self::Str(_) => Some(Type::Str),
            _ => None
        }
    }
}
impl fmt::Display for IToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            IToken::Bln(b) => b.to_string(),
            IToken::Flt(f) => f.to_string(),
            IToken::Fnc(s, _) => format!("{}(...)", s.name),
            IToken::Int(i) => i.to_string(),
            IToken::LParen => "(".to_string(),
            IToken::Mcr(s, _) => format!("{}!(...)", s),
            IToken::Oper(o) => o.to_str().to_string(),
            IToken::RParen => ")".to_string(),
            IToken::Scr(s) => s.get_mcname(),
            IToken::Str(s) => s.clone()
        })
    }
}
impl FToken {
    fn i_tokenize(self, compiler:&Compiler) -> Result<IToken, CompileError> {
        match self {
            Self::Int(i) => Ok(IToken::Int(i)),
            Self::Flt(f) => Ok(IToken::Flt(f)),
            Self::Bln(b) => Ok(IToken::Bln(b)),
            Self::Str(s) => Ok(IToken::Str(s)),
            Self::Scr(s) => Ok(IToken::Scr(
                compiler
                    .get_score(&s)
                    .ok_or(CompileError::UndefinedIdentifierReferenced(s))?
                    .clone()
            )),
            Self::Fnc(f, a) => Ok(IToken::Fnc(
                compiler
                    .get_func(&f)
                    .ok_or(CompileError::UndefinedIdentifierReferenced(f))?
                    .clone(),
                a.clone()
            )),
            Self::Mcr(m, a) => Ok(IToken::Mcr(m, a)),
            Self::Oper(o) => Ok(IToken::Oper(o)),
            Self::LParen => Ok(IToken::LParen),
            Self::RParen => Ok(IToken::RParen)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MCFunction {
    pub name: String,
    pub inside: String,
    pub path: Vec<String>,
    callment_prefix: String,
    preprocess: String,
    postprocess: String,
    returning_type: Type
}
impl MCFunction {
    fn call(&self, compiler:&Compiler) -> String {
        format!(
            "{}\n{} function {}/{}",
            self.postprocess,
            self.callment_prefix,
            compiler.namespace,
            self.path.join("/")
        )
    }
}
pub trait MCFunctionizable {
    fn mcfunctionate(&self, compiler:&mut Compiler) -> Result<MCFunction, CompileError>;
}
impl Serialisable for AST {
    fn serialise(&self, compiler:&mut Compiler) -> Result<Vec<CommandAST>, CompileError> {
        match self {
            AST::CodeBlock(_) | AST::IfSyntax(_) | AST::WhileSyntax(_) => {
                let mcfunctionated = self.mcfunctionate(compiler)?;
                let callment = mcfunctionated.call(compiler);
                compiler.compiled.push(mcfunctionated);
                Ok(vec![CommandAST::Native(callment)])
            },
            AST::Assignment(a) => a.serialise(compiler),
            AST::Formula(f) => f.serialise(compiler),
            AST::LetStatement(l) => l.serialise(compiler)
        }
    }
}

fn c_ast_to_string(c_ast:&Vec<CommandAST>) -> String {
    c_ast.iter().map(|t| t.serialise()).collect::<Vec<String>>().join("\n")
}
impl MCFunctionizable for CodeBlock {
    fn mcfunctionate(&self, compiler:&mut Compiler) -> Result<MCFunction, CompileError> {
        let mut serialised = Vec::new();
        for ast in self.inside.iter() {
            serialised.extend(ast.serialise(compiler)?);
        }
        let mut result = Vec::new();
        for c_ast in serialised {
            result.push(c_ast.serialise())
        }
        Ok(MCFunction {
            name: generate_random_id(32),
            inside: result.join("\n"),
            path: compiler.scope.clone(),
            callment_prefix: String::new(),
            preprocess: String::new(),
            postprocess: String::new(),
            returning_type: Type::None
        })
    }
}
impl MCFunctionizable for IfSyntax {
    fn mcfunctionate(&self, compiler:&mut Compiler) -> Result<MCFunction, CompileError> {
        let mut mcfunction = self.block.mcfunctionate(compiler)?;
        let condition_reserv = self.condition.to_calc_reserv(compiler)?;
        let is_true = Scoreboard {
            name: format!("IF_CONDITION_{}", scoreboard::generate_random_id(32)),
            scope: compiler.scope.clone(),
            datatype: condition_reserv.guess_type()?
        };
        let zero_const = Scoreboard {
            name: format!("0"),
            scope: vec!["CONST".to_string()],
            datatype: Type::Int
        };

        let mut preprocess = condition_reserv.to_be(&is_true)?;
        preprocess.extend(
            FormulaConstructer
                ::new()
                .assign_num(&zero_const, 0)
                .build()
        );
        mcfunction.preprocess = c_ast_to_string(&preprocess);
        mcfunction.callment_prefix = ExecuteConstructer
            ::new()
            .compare(
                &is_true,
                &Comparison::Neq,
                &zero_const
            ).build();
        mcfunction.postprocess = c_ast_to_string(
            &FormulaConstructer
                ::new()
                .free(&is_true)
                .free(&zero_const)
                .build()
        );
        Ok(mcfunction)
    }
}
impl MCFunctionizable for WhileSyntax {
    fn mcfunctionate(&self, compiler:&mut Compiler) -> Result<MCFunction, CompileError> {
        let mut codeblock = self.block.mcfunctionate(compiler)?;
        let is_true = Scoreboard {
            name: format!("WHILE_CONDITION_{}", scoreboard::generate_random_id(32)),
            scope: compiler.scope.clone(),
            datatype: Type::Bool
        };
        let zero_const = Scoreboard {
            name: format!("0"),
            scope: vec!["CONST".to_string()],
            datatype: Type::Int
        };
        
        let mut eval = self
            .condition
            .to_calc_reserv(compiler)?
            .to_be(&is_true)?;
        eval.extend(FormulaConstructer
            ::new()
            .assign_num(&zero_const, 0)
            .build()
        );
        eval.push(CommandAST::Native(format!(
            "{} {}",
            ExecuteConstructer::new()
                .compare(&is_true,&Comparison::Neq,&zero_const).build(),
            codeblock.call(compiler)
        )));
        eval.extend(FormulaConstructer
            ::new()
            .free(&is_true)
            .free(&zero_const)
            .build()
        );
        codeblock.postprocess = c_ast_to_string(&eval);
        Ok(codeblock)
    }
}

trait Serialisable {
    fn serialise(&self, compiler:&mut Compiler) -> Result<Vec<CommandAST>, CompileError>;
}

impl MCFunctionizable for AST {
    fn mcfunctionate(&self, compiler:&mut Compiler) -> Result<MCFunction, CompileError> {
        match self {
            AST::CodeBlock(c) => c.mcfunctionate(compiler),
            AST::IfSyntax(i) => i.mcfunctionate(compiler),
            AST::WhileSyntax(w) => w.mcfunctionate(compiler),
            _ => CodeBlock { inside: vec![self.clone()] }.mcfunctionate(compiler)
        }
    }
}

impl Serialisable for VariableDefinement {
    fn serialise(&self, compiler:&mut Compiler) -> Result<Vec<CommandAST>, CompileError> {
        let score = Scoreboard {
            name: self.identifier.clone(),
            scope: compiler.scope.clone(),
            datatype: match self.datatype {
                Some(s) => s,
                None => match &self.initialise {
                    Some(s) => s.rhs.to_calc_reserv(compiler)?.guess_type()?,
                    None => Err(CompileError::TheTypeOfAIndentifierWontBeConfirmed(self.identifier.clone()))?
                }
            }
        };
        let cast = match &self.initialise {
            Some(s) => s.rhs
                .to_calc_reserv(compiler)?
                .serialise(&score)?,
            None => Vec::new()
        };
        compiler.variables.push(score);
        Ok(cast)
    }
}
struct CalcReserv {
    tokens: Vec<IToken>
}
impl From<Vec<IToken>> for CalcReserv {
    fn from(value: Vec<IToken>) -> Self {
        CalcReserv { tokens: value }
    }
}
impl CalcReserv {
    fn serialise(&self, store_to:&Scoreboard) -> Result<Vec<CommandAST>, CompileError> {
        let mut commands: Vec<CommandAST> = Vec::new();
        let mut stack: Vec<IToken> = Vec::new();
        let mut temp_scores:Vec<Scoreboard> = Vec::new();

        for token in &self.tokens {
            // Move values to the stack
            if token.is_value() {
                match token {
                    IToken::Scr(_) | IToken::Fnc(_, _) | IToken::Mcr(_, _) | IToken::Int(_) | IToken::Flt(_) | IToken::Bln(_) => {
                        stack.push(token.clone());
                    },
                    _ => return Err(CompileError::UnsupportedLiteralType(token.clone())),
                }
            // Calcate if a operator poped
            } else if let IToken::Oper(operator) = token {
                // Get lhs and rhs
                let rhs_board = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;
                let lhs_board = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;
                // A calc result expect a container scoreboard
                let result_container = Scoreboard {
                    name: format!("CALC_RESULT_{}", scoreboard::generate_random_id(scoreboard::TEMP_ID_LEN)),
                    scope: vec!["TEMP".to_string()],
                    datatype: lhs_board.get_datatype().unwrap()
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
                stack.push(IToken::Scr(result_container));
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
    fn guess_type(&self) -> Result<Type, CompileError> {
        let mut stack: Vec<Type> = Vec::new();
        for token in &self.tokens {
            // Move values to the stack
            if token.is_value() {
                match token {
                    IToken::Scr(_) | IToken::Fnc(_, _) | IToken::Mcr(_, _) | IToken::Int(_) | IToken::Flt(_) | IToken::Bln(_) => {
                        stack.push(token.get_datatype().unwrap());
                    },
                    _ => return Err(CompileError::UnsupportedLiteralType(token.clone())),
                }
            // Calcate if a operator poped
            } else if let IToken::Oper(operator) = &token {
                // Get lhs and rhs
                let rhs = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;
                let lhs = stack.pop().ok_or(CompileError::InvalidFormulaStructure("Not enough operands for operator".to_string()))?;
                stack.push(match operator {
                    Oper::Arithmetic(a) => a
                        .get_type(&lhs, &rhs)
                        .ok_or(CompileError::UndefinedOperation(
                            lhs.clone(), operator.clone(), rhs.clone()
                        ))?,
                    Oper::Comparison(b) => b
                        .get_type(&lhs, &rhs)
                        .ok_or(CompileError::UndefinedOperation(
                            lhs.clone(), operator.clone(), rhs.clone()
                        ))?,
                    Oper::Logical(l) => l
                        .get_type(&lhs, &rhs)
                        .ok_or(CompileError::UndefinedOperation(
                            lhs.clone(), operator.clone(), rhs.clone()
                        ))?,
                })
            } else {
                // There's no token that isn't value nor operator in formula token, right?
                unreachable!("Non-operator, non-value token found in RPN: {:?}", token);
            }
        }
        if stack.len() == 1 {
            Ok(stack.pop().unwrap())
        } else {
            return Err(CompileError::UnbalancedParentheses);
        }
    }
    fn to_be(&self, store_to:&Scoreboard) -> Result<Vec<CommandAST>, CompileError> {
        let evaluated_stored = Scoreboard {
            name: format!("TO_BE_{}", scoreboard::generate_random_id(32)),
            scope: vec!["TEMP".to_string()],
            datatype: self.guess_type()?
        };
        let mut result = self.serialise(&evaluated_stored)?;
        result.extend(FormulaConstructer
            ::new()
            .boolify_num_comparison(
                &evaluated_stored,
                Comparison::Neq.to_str().to_string(),
                0
            )
            .assign_score(store_to, &evaluated_stored)
            .free(&evaluated_stored)
            .build()
        );
        Ok(result)
    }
}
impl RPNFormula {
    fn to_calc_reserv(&self, compiler:&Compiler) -> Result<CalcReserv, CompileError> {
        let mut i_tokenized = Vec::new();
        for t in &self.formula_tokens {
            i_tokenized.push(t.clone().i_tokenize(compiler)?);
        }
        Ok(CalcReserv::from(i_tokenized))
    }
}

impl Serialisable for Assignment {
    fn serialise(&self, compiler:&mut Compiler) -> Result<Vec<CommandAST>, CompileError> {
        let store_to = compiler
            .get_score(&self.lhs)
            .ok_or(CompileError::UndefinedIdentifierReferenced(self.lhs.clone()))?;
        self.rhs.to_calc_reserv(compiler)?.serialise(store_to)
    }
}
impl Serialisable for Formula {
    fn serialise(&self, compiler:&mut Compiler) -> Result<Vec<CommandAST>, CompileError> {
        let rpn = match self.clone().to_rpn() {
            Ok(o) => o,
            Err(e) => Err(CompileError::ASyntaxErrorOccured(e.clone()))?
        }.to_calc_reserv(compiler)?;
        let store_to = scoreboard::get_calc_result_temp(
            rpn.guess_type()?
        );
        rpn.serialise(&store_to)
    }
}