use std::collections::VecDeque;

use crate::tokenizer::Token;

#[derive(Debug)]
pub struct Ident(pub String);

#[derive(Debug)]
pub enum Argument {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    OptionalInteger(Option<i64>),
    OptionalFloat(Option<f64>),
    OptionalString(Option<String>),
    OptionalBoolean(Option<bool>),
}

impl TryFrom<Argument> for i64 {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) => Ok(i),
            _ => return Err(format!("expected `Integer` got {:?}", arg).into()),
        }
    }
}

impl TryFrom<Argument> for f64 {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) => Ok(i as f64),
            Argument::Float(f) => Ok(f),
            _ => return Err(format!("expected `Float` got {:?}", arg).into()),
        }
    }
}

impl TryFrom<Argument> for String {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) => Ok(i.to_string()),
            Argument::Float(f) => Ok(f.to_string()),
            Argument::String(s) => Ok(s),
            Argument::Boolean(b) => Ok(b.to_string()),
            _ => return Err(format!("expected `String` got {:?}", arg).into()),
        }
    }
}

impl TryFrom<Argument> for bool {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) if i == 0 => Ok(false),
            Argument::Integer(i) if i == 1 => Ok(true),
            Argument::Boolean(b) => Ok(b),
            _ => return Err(format!("expected `Boolean` got {:?}", arg).into()),
        }
    }
}

impl TryFrom<Argument> for Option<i64> {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) => Ok(Some(i)),
            Argument::OptionalInteger(i) => Ok(i),
            _ => return Err(format!("expected `OptionalInteger` got {:?}", arg).into()),
        }
    }
}

impl TryFrom<Argument> for Option<f64> {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) => Ok(Some(i as f64)),
            Argument::Float(f) => Ok(Some(f)),
            Argument::OptionalFloat(f) => Ok(f),
            _ => return Err(format!("expected `OptionalFloat` got {:?}", arg).into()),
        }
    }
}

impl TryFrom<Argument> for Option<String> {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) => Ok(Some(i.to_string())),
            Argument::Float(f) => Ok(Some(f.to_string())),
            Argument::String(s) => Ok(Some(s)),
            Argument::Boolean(b) => Ok(Some(b.to_string())),
            Argument::OptionalString(s) => Ok(s),
            _ => return Err(format!("expected `OptionalString` got {:?}", arg).into()),
        }
    }
}

impl TryFrom<Argument> for Option<bool> {
    type Error = Box<dyn std::error::Error>;
    fn try_from(arg: Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Integer(i) if i == 0 => Ok(Some(false)),
            Argument::Integer(i) if i == 1 => Ok(Some(true)),
            Argument::Boolean(b) => Ok(Some(b)),
            Argument::OptionalBoolean(b) => Ok(b),
            _ => return Err(format!("expected `OptionalBoolean` got {:?}", arg).into()),
        }
    }
}

pub fn parse(input: &str) -> Result<(Ident, VecDeque<Argument>), Box<dyn std::error::Error>> {
    let mut tokens = crate::tokenizer::tokenize(input);

    let mut args = VecDeque::new();

    let cmd: Ident = match tokens.pop_front() {
        Some(Token::UnquotedString(s)) => Ident(s),
        _ => return Err("Invalid command".into()),
    };

    while let Some(token) = tokens.pop_front() {
        match token {
            Token::Integer(i) => args.push_back(Argument::Integer(i)),
            Token::Float(f) => args.push_back(Argument::Float(f)),
            Token::String(s) => args.push_back(Argument::String(s)),
            Token::Boolean(b) => args.push_back(Argument::Boolean(b)),
            Token::UnquotedString(s) => args.push_back(Argument::String(s)),
        }
    }

    Ok((cmd, args))
}
