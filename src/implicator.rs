use crate::tokeniser::Token;

enum ImplicateError {
    UnknownIdentifier(String),
    EmptyWasGiven,
    SentenceWasntClosed,
}
enum Line {
    Sentense(Sentense),
    Formula(Formula)
}
struct Sentense {
    specialiser : Option<(Token, Vec<Token>)>,
    inside : Vec<Line>
}
struct Formula {
    lhs : Option<Token>,
    rhs : Vec<Token>
}

pub fn implicate(given:Vec<Token>) -> Result<Sentense, ImplicateError> {
    let mut tokens = given.iter().peekable();
    let mut token = match tokens.next() {
        Some(s) => *s,
        None => return Err(ImplicateError::EmptyWasGiven)
    };
    let mut queue:Vec<Token>;
    'main: loop {
        let result = match token {
            Token::LBrace => {
                let specifier = if queue.is_empty() {
                    None
                } else {
                    let 
                };
                let mut inside:Vec<Token> = Vec::new();
                while token == Token::RBrace {
                    token = match tokens.next() {
                        Some(s) => s,
                        None => return Err(ImplicateError::SentenceWasntClosed)
                    }
                }
                let sentence = Sentense {
                    specialiser : specifier,
                    inside : inside
                };
                Line(sentence)
            },
            Token::Semicolon => {
                if queue.contains(&Token::Asn) {

                }
            }
            _ => { queue.push(token); }
        };
    }
}