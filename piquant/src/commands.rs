use piquant_macros::command;
use valence::prelude::{Client, World};

use crate::server::Game;

/// A simple test command
/// * `str`: An optional string to print
#[command]
pub fn test(client: Client<Game>, str: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, {}!", client.username());

    if let Some(str) = str {
        client.send_message(format!("You tested for: {}", str));
    }

    Ok(())
}

#[command]
pub fn seed(client: Client<Game>, world: World<Game>) -> Result<(), Box<dyn std::error::Error>> {
    let seed: u32 = world.state.seed.clone().unwrap().into();

    client.send_message(format!("World Seed: {}", seed));

    Ok(())
}
