#[derive(Debug, PartialEq)]
pub enum Token {
    // Special Tokens
    Eof,

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

const WHITESPACE:[char; 4] = [' ', '\n', '\t', '\r'];
const NUMERIC:[char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

fn delimiter(input:&char) -> Option<Token> {
    match input  {
        '.' => Some(Token::Dot),
        ',' => Some(Token::Comma),
        ';' => Some(Token::Semicolon),
        ':' => Some(Token::Colon),
        '(' => Some(Token::LParen),
        ')' => Some(Token::RParen),
        '{' => Some(Token::LBrace),
        '}' => Some(Token::RBrace),
        '[' => Some(Token::LBracket),
        ']' => Some(Token::RBracket),
        _   => None
    }
}
fn operator(current_char:&char, next_char:&char) -> Option<Token> {
    // Two chars
    match format!("{}{}", current_char, next_char).as_str() {
        // Arrows
        "->" => return Some(Token::Arr),
        "=>" => return Some(Token::FArr),

        // Compare
        "==" => return Some(Token::Eq),
        "!=" => return Some(Token::NEq),
        "<=" => return Some(Token::LEt),
        ">=" => return Some(Token::REt),

        _ => ()
    };
    // Single char
    match current_char {
        // Operator
        '=' => return Some(Token::Asn),
        '+' => return Some(Token::Add),
        '-' => return Some(Token::Rem),
        '*' => return Some(Token::Mul),
        '/' => return Some(Token::Div),
        '%' => return Some(Token::Sur),

        // Compare
        '<' => return Some(Token::Lt),
        '>' => return Some(Token::Gt),

        // Logical Operator
        '!' => return Some(Token::Neg),
        '|' => return Some(Token::Or),
        '&' => return Some(Token::And),

        _   => ()
    }
    None
}
/// It returns identifier or keyword
fn solve_a_word(input:&str) -> Token {
    match input {
        "let"   => Token::Let,
        "fn"    => Token::Fn,
        "if"    => Token::If,
        "else"  => Token::Else,
        "while" => Token::While,
        "for"   => Token::For,
        "int"   => Token::IntType,
        "float" => Token::FltType,
        "bool"  => Token::BlnType,
        "none"  => Token::NoneType,
        "return"=> Token::Return,
        "true"  => Token::Bln(true),
        "false" => Token::Bln(false),
        _       => Token::Ident(input.to_string())
    }
}

#[test]
fn tokenizer_test() {
    println!("{:?}", tokenize("fn main -> int {let a = 1.14 + 5.14; return 810}".to_string()))
}
pub fn tokenize(input:String) -> Vec<Token> {
    let mut queue = String::new();
    let mut chars = input.chars().peekable();
    let mut tokens:Vec<Token> = Vec::new();

    'main: loop {
        let mut cur_char = match chars.next() {
            Some(s) => s,
            None => {break 'main;}
        };
        
        // Skip whitespaces
        if WHITESPACE.contains(&cur_char) {
            continue 'main;
        }
        // Parse a delimiter
        if let Some(s) = delimiter(&cur_char) {
            tokens.push(s);
            continue 'main;
        }
        // Parse a operator
        if let Some(s) = operator(&cur_char, chars.peek().unwrap()) {
            tokens.push(s);
            continue 'main;
        }
        // Parse a string literal
        if cur_char == '"' {
            let mut inside = String::new();
            loop {
                if cur_char == '"' {
                    break;
                }
                inside.push(cur_char);
                cur_char = match chars.next() {
                    Some(s) => s,
                    None => { break 'main }
                };
            }
            tokens.push(Token::Str(inside));
            continue 'main;
        }
        // Parse a mcid literal
        if cur_char == '$' {
            let mut mc_id = String::new();
            loop {
                if WHITESPACE.contains(&cur_char) {
                    break;
                }
                mc_id.push(cur_char);
                cur_char = match chars.next() {
                    Some(s) => s,
                    None => { break 'main }
                };
            }
            tokens.push(Token::MCId(mc_id));
            continue 'main;
        }
        // Parse a int/float literal
        if NUMERIC.contains(&cur_char) {
            let mut numeric = String::new();
            let mut is_float = false;
            numeric.push(cur_char);

            loop {
                // Goto next char
                let next = match chars.peek() {
                    Some(s) => s,
                    None => { break 'main }
                };
                // Eval a charactor
                if NUMERIC.contains(next) {
                    numeric.push(chars.next().unwrap());
                } else if next == &'.' {
                    numeric.push(chars.next().unwrap());
                    is_float = true;
                } else {
                    break;
                }
            }
            if is_float {
                tokens.push(Token::Flt(numeric.parse().unwrap()));
            } else {
                tokens.push(Token::Int(numeric.parse().unwrap()));
            }
            continue 'main;
        }
        // Solve identifier/keyword
        queue.push(cur_char);
        
        match chars.peek() {
            Some(c) => {
                if WHITESPACE.contains(c) {
                    tokens.push(solve_a_word(&queue));
                    queue.clear();
                }
                continue 'main;
            },
            None => {break 'main}
        }
    }
    tokens.push(Token::Eof);
    tokens
}