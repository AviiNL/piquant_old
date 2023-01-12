use valence_protocol::{
    packets::s2c::commands::{Node, NodeData, Parser, StringArg},
    VarInt,
};

use crate::CommandDef;

pub struct CommandStack {
    commands: Vec<Node<'static>>,
    command_ids: Vec<i32>,
}

impl CommandStack {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            command_ids: Vec::new(),
        }
    }

    fn add(&mut self, node: Node<'static>) -> i32 {
        self.commands.push(node);
        self.commands.len() as i32 - 1
    }

    pub fn register(&mut self, command_def: CommandDef) {
        let name = command_def.name;
        let args = command_def.arguments;

        let mut arg_ids = Vec::new();
        // loop over args
        for arg in args {
            let parser = match arg.ty {
                "i64" => Parser::Integer {
                    min: None,
                    max: None,
                },
                "f64" => Parser::Float {
                    min: None,
                    max: None,
                },
                "bool" => Parser::Bool,
                "string" => Parser::String(StringArg::QuotablePhrase),
                _ => Parser::String(StringArg::QuotablePhrase),
            };

            let a = Node {
                children: Vec::new(),
                data: NodeData::Argument {
                    name: &arg.name[..],
                    parser,
                    suggestion: None,
                },
                executable: true,
                redirect_node: None,
            };

            let arg_id = self.add(a);
            arg_ids.push(arg_id);
        }

        let children = arg_ids.iter().map(|id| VarInt(*id)).collect();

        let command = Node {
            children,
            data: NodeData::Literal { name },
            executable: true,
            redirect_node: None,
        };

        let id = self.add(command);

        self.command_ids.push(id);
    }

    pub fn list(&self) -> (i32, Vec<Node>) {
        let mut cloned_commands = self.commands.clone();

        let root_node = Node {
            children: self.command_ids.iter().map(|id| VarInt(*id)).collect(),
            data: NodeData::Root,
            executable: false,
            redirect_node: None,
        };

        cloned_commands.push(root_node);

        (cloned_commands.len() as i32 - 1, cloned_commands)
    }
}
