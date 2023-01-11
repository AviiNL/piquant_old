use valence::prelude::Client;

use crate::server::Game;

#[piquant_macros::command]
pub fn test(client: Client<Game>, str: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, {}!", client.username());

    if let Some(str) = str {
        client.send_message(format!("You tested for: {}", str));
    }

    Ok(())
}
