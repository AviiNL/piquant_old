mod command_service;
mod parser;
mod tokenizer;

use std::collections::VecDeque;

pub use command_service::CommandService;

pub use parser::parse;
pub use parser::Argument;

pub type Arguments = VecDeque<Argument>;
pub type Command<C> = fn(Arguments, &mut C) -> Result<(), Box<dyn std::error::Error>>;
