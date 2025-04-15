use super::Token;

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
fn operator(current_char:&char, next_char:Option<&char>) -> Option<(Token, bool)> {
    // Two chars
    if let Some(next) = next_char {
        match match format!("{}{}", current_char, next).as_str() {
            // Arrows
            "->" => Some(Token::Arr),
            "=>" => Some(Token::FArr),

            // Compare
            "==" => Some(Token::Eq),
            "!=" => Some(Token::NEq),
            "<=" => Some(Token::LEt),
            ">=" => Some(Token::REt),

            _ => None
        } {
            Some(s) => return Some((s, true)),
            None => ()
        }
    }
    // Single char
    match match current_char {
        // Operator
        '=' => Some(Token::Asn),
        '+' => Some(Token::Add),
        '-' => Some(Token::Rem),
        '*' => Some(Token::Mul),
        '/' => Some(Token::Div),
        '%' => Some(Token::Sur),

        // Compare
        '<' => Some(Token::Lt),
        '>' => Some(Token::Gt),

        // Logical Operator
        '!' => Some(Token::Neg),
        '|' => Some(Token::Or),
        '&' => Some(Token::And),

        _   => None
    } {
        Some(s) => return Some((s, false)),
        None => None
    }
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
    println!("{:?}", tokenize("fn main -> int {let a = 1.14 + 5.14; return 810;}".to_string()))
}
pub fn tokenize(input:String) -> Vec<Token> {
    let mut queue = String::new();
    let mut chars = input.chars().peekable();
    let mut tokens:Vec<Token> = Vec::new();

    let flush_queue = |q: &mut String, ts: &mut Vec<Token>| {
        if !q.is_empty() {
            ts.push(solve_a_word(q));
            q.clear();
        }
    };

    while let Some(cur_char) = chars.peek().cloned() { // peekして、処理後にnextする戦略

        // 1. 空白文字か？
        if WHITESPACE.contains(&cur_char) {
            flush_queue(&mut queue, &mut tokens); // 空白の前が識別子なら確定
            chars.next(); // 空白を消費
            continue;
        }

        // 2. 区切り文字か？
        if let Some(token) = delimiter(&cur_char) {
            flush_queue(&mut queue, &mut tokens); // 区切り文字の前が識別子なら確定
            tokens.push(token);
            chars.next(); // 区切り文字を消費
            continue;
        }

        // 3. 演算子か？ (次の文字も考慮)
        let next_char_peek = chars.clone().nth(1); // peek().peek() のようなこと
        if let Some((token, consumed_next)) = operator(&cur_char, next_char_peek.as_ref()) {
            flush_queue(&mut queue, &mut tokens); // 演算子の前が識別子なら確定
            tokens.push(token);
            chars.next(); // 演算子の最初の文字を消費
            if consumed_next {
                chars.next(); // 演算子の2番目の文字も消費
            }
            continue;
        }

        // 4. 文字列リテラルか？ (")
        if cur_char == '"' {
            flush_queue(&mut queue, &mut tokens); // 文字列リテラルの前が識別子なら確定 (通常はないはずだが念のため)
            chars.next(); // 開始の " を消費
            let mut inside = String::new();
            let mut closed = false;
            while let Some(string_char) = chars.next() {
                if string_char == '"' {
                    closed = true;
                    break; // 終了の " を見つけたらループを抜ける
                }
                inside.push(string_char);
            }
            if !closed {
                // エラー処理: 文字列が閉じられていない
                println!("Warning: Unclosed string literal encountered.");
                // tokens.push(Token::Error("Unclosed string literal".to_string()));
            }
            tokens.push(Token::Str(inside));
            continue;
        }

        // 5. MCID リテラルか？ ($) - $の後の識別子を読む想定
        if cur_char == '$' {
            flush_queue(&mut queue, &mut tokens); // MCIDの前が識別子なら確定
            chars.next(); // '$' を消費
            let mut mc_id = String::new();
            while let Some(next_peek) = chars.peek() {
                 // 次が区切り文字や演算子でなく、識別子に使える文字なら続ける
                 // (ここでは単純化のため空白、デリミタ、演算子でないかで判断)
                if !WHITESPACE.contains(next_peek) &&
                    delimiter(next_peek).is_none() &&
                    operator(next_peek, None).is_none() && // 1文字演算子でない
                    next_peek != &'"' && next_peek != &'$' // 他のリテラル開始文字でもない
                {
                    mc_id.push(chars.next().unwrap()); // 文字を消費して追加
                } else {
                    break; // 区切りが見えたら終了
                }
            }
            if !mc_id.is_empty() {
                tokens.push(Token::MCId(mc_id));
            } else {
                 // エラー処理: '$' の後に識別子が続かない
                println!("Warning: '$' not followed by an identifier.");
                 // tokens.push(Token::Error("Expected identifier after $".to_string()));
            }
            continue;
        }

        // 6. 数値リテラルか？ (数字で始まる)
        if NUMERIC.contains(&cur_char) {
            flush_queue(&mut queue, &mut tokens); // 数値の前が識別子なら確定 (通常はない)
            let mut numeric = String::new();
            let mut is_float = false;
            numeric.push(chars.next().unwrap()); // 最初の数字を消費

            while let Some(next_peek) = chars.peek() {
                if NUMERIC.contains(next_peek) {
                    numeric.push(chars.next().unwrap());
                } else if *next_peek == '.' {
                     // '.' の次が数字かどうかで float か判断
                     let mut temp_chars = chars.clone(); // イテレータをコピーして先読み
                     temp_chars.next(); // '.' を仮想的に消費
                    if temp_chars.peek().map_or(false, |c| NUMERIC.contains(c)) {
                          // '.' の次が数字なら float
                          if is_float { // すでに '.' が含まれている場合 (例: 1.2.3)
                                println!("Warning: Multiple '.' in numeric literal '{}'", numeric);
                               break; // 不正な数値としてここで終了
                        }
                          numeric.push(chars.next().unwrap()); // '.' を消費して追加
                        is_float = true;
                    } else {
                        // '.' の次が数字でない -> '.' は数値の一部ではない
                        break;
                    }
                } else {
                     // 数字でも '.' でもないなら数値リテラル終了
                    break;
                }
            }

            if is_float {
                match numeric.parse::<f32>() {
                    Ok(f) => tokens.push(Token::Flt(f)),
                    Err(e) => println!("Error parsing float '{}': {}", numeric, e),
                    // tokens.push(Token::Error(format!("Invalid float: {}", numeric)))
                }
            } else {
                match numeric.parse::<i32>() {
                    Ok(i) => tokens.push(Token::Int(i)),
                    Err(e) => println!("Error parsing int '{}': {}", numeric, e),
                    // tokens.push(Token::Error(format!("Invalid int: {}", numeric)))
                }
            }
            continue;
        }

        // 7. 上記のいずれでもなければ、識別子/キーワードの一部
        // (cur_char は peek() で見ただけなのでここで消費する)
        queue.push(chars.next().unwrap());

    } // while let Some(cur_char) = chars.peek().cloned()

    // ループ終了後、queue に残っている最後の識別子/キーワードを処理
    flush_queue(&mut queue, &mut tokens);

    tokens
}