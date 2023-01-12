use std::collections::HashMap;

use crate::{parse, Command};

pub struct CommandService<C, W> {
    commands: HashMap<String, Command<C, W>>,
}

impl<C, W> CommandService<C, W> {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn add_command(&mut self, name: &str, command: Command<C, W>) {
        self.commands.insert(name.to_string(), command);
    }

    pub fn execute(
        &self,
        input: &str,
        client: &mut C,
        world: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (cmd, args) = parse(input)?;

        let mut cmd = cmd.0;
        if cmd.starts_with('/') {
            cmd.remove(0);
        }

        // args.push_back(Argument::Client(client));

        if let Some(command) = self.commands.get(&cmd) {
            command(args, client, world)
        } else {
            Err("Command not found".into())
        }
    }
}
