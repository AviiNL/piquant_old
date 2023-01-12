mod command_service;
mod parser;
mod tokenizer;

use std::collections::VecDeque;

pub use command_service::CommandService;

pub use parser::parse;
pub use parser::Argument;

pub type Arguments = VecDeque<Argument>;
pub type Command<G, C, W> =
    fn(Arguments, &G, &mut C, &mut W) -> Result<(), Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct ArgumentDef {
    pub name: String,
    pub ty: String,
    pub optional: bool,
}

#[derive(Debug)]
pub struct CommandDef {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<ArgumentDef>,
}
