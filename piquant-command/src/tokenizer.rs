use std::collections::VecDeque;

#[derive(Debug)]
pub enum Token {
    Integer(i64),
    Float(f64),
    String(String),
    UnquotedString(String),
    Boolean(bool),
}

pub(crate) fn tokenize(input: &str) -> VecDeque<Token> {
    let mut tokens = VecDeque::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Strings are either single word, or surrounded by unescaped double quotes.
            '"' => {
                let mut string = String::new();

                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => {
                            if let Some(c) = chars.next() {
                                string.push(c);
                            }
                        }
                        _ => string.push(c),
                    }
                }

                tokens.push_back(Token::String(string));
            }
            // Integers and floats are a sequence of digits and optionally a single decimal point.
            '0'..='9' | '.' => {
                let mut number = String::new();
                number.push(c);

                while let Some(c) = chars.peek() {
                    match c {
                        ' ' => break,
                        cc => {
                            number.push(*cc);
                            chars.next();
                        }
                    }
                }

                if number.contains('.') {
                    if let Ok(number) = number.parse::<f64>() {
                        tokens.push_back(Token::Float(number));
                    }
                } else if let Ok(number) = number.parse::<i64>() {
                    tokens.push_back(Token::Integer(number));
                } else {
                    tokens.push_back(Token::UnquotedString(number));
                }
            }
            // Booleans are either "true" or "false".
            'a'..='z' | 'A'..='Z' => {
                let mut value = String::new();
                value.push(c);

                while let Some(c) = chars.peek() {
                    match c {
                        ' ' => {
                            break;
                        }
                        cc => {
                            value.push(*cc);
                            chars.next();
                        }
                    }
                }

                match value.as_str() {
                    "true" => tokens.push_back(Token::Boolean(true)),
                    "false" => tokens.push_back(Token::Boolean(false)),
                    _ => tokens.push_back(Token::UnquotedString(value)),
                }
            }
            // Ignore whitespace.
            _ => (),
        }
    }

    tokens
}
