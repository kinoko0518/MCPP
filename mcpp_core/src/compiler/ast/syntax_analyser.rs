use super::*;

use crate::compiler::Oper;
use crate::compiler::Token;
use crate::compiler::ast;

use crate::evaluater::scoreboard::arithmetic_operation::Arithmetic;
use crate::evaluater::scoreboard::logical_operation::Logical;
use crate::evaluater::scoreboard::comparison_operation::Comparison;

#[test]
fn test() {
    let token = vec![
        Token::LBrace, Token::If, Token::Ident("a".to_string()), Token::Gt, Token::Int(10), Token::LBrace,
        Token::Let
    ];
}

pub struct SyntaxAnalyser {
    tokens: Vec<Token>,
}
fn expect_token_err(name:&str) -> SyntaxError {
    SyntaxError::ExpectedAToken(name.to_string())
}

impl From<Vec<Token>> for SyntaxAnalyser {
    fn from(value: Vec<Token>) -> Self {
        SyntaxAnalyser { tokens: value }
    }
}
impl SyntaxAnalyser {
    fn get_locally(&self, gap:usize) -> Option<&Token> {
        self.tokens.get(gap)
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(0)
    }
    fn consume(&mut self) -> Option<Token> {
        if self.tokens.is_empty() {
            None
        } else {
            Some(self.tokens.remove(0))
        }
    }
    fn expect(&mut self) -> Result<Token, SyntaxError> {
        self.consume().ok_or(SyntaxError::TokenEndsUnexpectedly)
    }
    fn get_tuple(&mut self) -> Result<Tuple, SyntaxError> {
        let syntax_error = SyntaxError::InvalidFormAs("tuple".to_string());
        if !matches!(self.expect()?, Token::LParen) {
            return Err(expect_token_err("("));
        }
        let mut formulas = Vec::new();
        loop {
            formulas.push(Formula::from(self.get_formula()?));
            match self.expect()? {
                Token::RParen => {break;},
                Token::Comma => {continue;},
                _ => {return Err(syntax_error);}
            }
        }
        Ok(Tuple { inside : formulas })
    }
    fn get_formula(&mut self) -> Result<Vec<FToken>, SyntaxError> {
        let mut queue = Vec::new();
        loop {
            let found = match self.consume() {
                Some(t) => match t {
                    Token::Int(i) => FToken::Int(i),
                    Token::Flt(f) => FToken::Flt(f),
                    Token::Bln(b) => FToken::Bln(b),
                    Token::Ident(i) => if let Some(t) = self.get_locally(1) {
                        match t {
                            Token::Neg => {
                                self.consume();
                                FToken::Mcr(i, self.get_tuple()?)
                            },
                            Token::LParen => FToken::Fnc(i, self.get_tuple()?),
                            _ => FToken::Scr(i)
                        }
                    } else {
                        FToken::Scr(i)
                    },
                    // Arithmetic operations
                    Token::Add => FToken::Oper(Oper::Arithmetic(Arithmetic::Add)),
                    Token::Rem => FToken::Oper(Oper::Arithmetic(Arithmetic::Rem)),
                    Token::Mul => FToken::Oper(Oper::Arithmetic(Arithmetic::Mul)),
                    Token::Div => FToken::Oper(Oper::Arithmetic(Arithmetic::Div)),
                    Token::Sur => FToken::Oper(Oper::Arithmetic(Arithmetic::Sur)),
                    
                    // Logical operations
                    Token::And => FToken::Oper(Oper::Logical(Logical::And)),
                    Token::Or  => FToken::Oper(Oper::Logical(Logical::Or)),
                    Token::Neg => FToken::Oper(Oper::Logical(Logical::Not)),

                    // Comparisons
                    Token::Gt  => FToken::Oper(Oper::Comparison(Comparison::Gt)),
                    Token::Lt  => FToken::Oper(Oper::Comparison(Comparison::Lt)),
                    Token::LEt => FToken::Oper(Oper::Comparison(Comparison::Le)),
                    Token::REt => FToken::Oper(Oper::Comparison(Comparison::Ge)),
                    Token::Eq  => FToken::Oper(Oper::Comparison(Comparison::Eq)),
                    Token::NEq => FToken::Oper(Oper::Comparison(Comparison::Neq)),

                    // Parentheses
                    Token::LParen => FToken::LParen,
                    Token::RParen => FToken::RParen,
                    _ => break
                },
                None => break
            };
            queue.push(found);
        }
        Ok(queue)
    }
    fn get_let(&mut self) -> Result<VariableDefinement, SyntaxError> {
        let syntax_error = SyntaxError::InvalidFormAs("let statement".to_string());
        if !matches!(self.expect()?, Token::Let) {
            return Err(SyntaxError::ExpectedAToken("let keyword".to_string()));
        }
        let identifier = if let Token::Ident(s) = self.expect()? {
            s
        } else {
            return Err(SyntaxError::ExpectedAToken("identifier".to_string()));
        };
        let initialise;
        let datatype;

        match self.expect()? {
            Token::Colon => {
                datatype = Some(
                    self
                        .consume()
                        .ok_or(SyntaxError::TokenEndsUnexpectedly)?
                        .to_type()
                        .ok_or(SyntaxError::ExpectedAToken("data type".to_string()))?
                );
                initialise = if let Token::Asn = self.expect()? {
                    Some(ast::Assignment {
                        lhs : identifier.clone(),
                        rhs : Formula::from(self.get_formula()?).to_rpn()?
                    })
                } else {
                    None
                };
            },
            Token::Asn => {
                let rhs = Formula::from(self.get_formula()?).to_rpn()?;
                datatype = None;
                initialise = Some(Assignment {
                    lhs : identifier.clone(),
                    rhs : rhs
                });
            },
            _ => {return Err(syntax_error);}
        }
        let var_definement = VariableDefinement {
            identifier : identifier,
            datatype : datatype,
            initialise : initialise
        };
        Ok(var_definement)
    }
    fn get_if(&mut self) -> Result<IfSyntax, SyntaxError> {
        if !matches!(self.expect()?, Token::If) {
            return Err(expect_token_err("if keyword"));
        }
        let conditon = Formula::from(self.get_formula()?).to_rpn()?;
        let inside = self.get_block()?;
        Ok(IfSyntax {
            condition: conditon,
            block: inside
        })
    }
    fn get_while(&mut self) -> Result<WhileSyntax, SyntaxError> {
        if !matches!(self.expect()?, Token::While) {
            return Err(expect_token_err("while keyword"));
        }
        let conditon = Formula::from(self.get_formula()?).to_rpn()?;
        let inside = self.get_block()?;
        Ok(WhileSyntax {
            condition: conditon,
            block: inside
        })
    }
    pub fn get_block(&mut self) -> Result<CodeBlock, SyntaxError> {
        let mut insides = Vec::new();
        if !matches!(self.expect()?, Token::LBrace) {
            return Err(expect_token_err("{"));
        }
        loop {
            let top_token = self.peek().ok_or(SyntaxError::UnbalancedBraces)?;
            let found = match top_token {
                Token::If => AST::IfSyntax(self.get_if()?),
                Token::While => AST::WhileSyntax(self.get_while()?),
                Token::Let => AST::LetStatement(self.get_let()?),
                Token::LBrace => AST::CodeBlock(self.get_block()?),
                Token::RBrace => {
                    break Ok(CodeBlock { inside: insides });
                },
                Token::Int(_) | Token::Bln(_) | Token::Flt(_) => AST::Formula(
                    Formula::from(self.get_formula()?)
                ),
                // identifier = ... or identifier [+-*/%] value ...
                Token::Ident(_) => if let Some(Token::Asn) = self.get_locally(1) {
                    AST::Assignment(self.get_assignment()?)
                } else {
                    AST::Formula(Formula::from(self.get_formula()?))
                },
                _ => return Err(SyntaxError::ALineMustntStartWith(top_token.clone()))
            };
            insides.push(found);
        }
    }
    fn get_assignment(&mut self) -> Result<Assignment, SyntaxError> {
        let syntax_error = SyntaxError::InvalidFormAs("assignment".to_string());
        let lhs = if let (Some(Token::Ident(s)), Some(Token::Asn)) = (self.consume(), self.consume()) {
            s
        } else {
            return Err(syntax_error);
        };
        let rhs = ast::Formula::from(self.get_formula()?).to_rpn()?;
        if !matches!(self.consume(), Some(Token::Semicolon)) {
            return Err(expect_token_err("semicolon"));
        }
        Ok(ast::Assignment {
            lhs : lhs,
            rhs : rhs
        })
    }
}