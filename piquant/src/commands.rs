use piquant_macros::command;
use valence::{
    prelude::{Chunk, ChunkPos, Client, Color, World},
    protocol::{BlockKind, BlockState, TextFormat},
};

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

#[command]
pub fn some_command(client: Client<Game>, data: String) -> Result<(), Box<dyn std::error::Error>> {
    client.send_message(format!("You sent: {}", data));

    Ok(())
}

#[command]
pub fn gamemode(client: Client<Game>, gamemode: String) -> Result<(), Box<dyn std::error::Error>> {
    let new_gamemode = match gamemode.as_str() {
        "survival" => valence::prelude::GameMode::Survival,
        "creative" => valence::prelude::GameMode::Creative,
        "adventure" => valence::prelude::GameMode::Adventure,
        "spectator" => valence::prelude::GameMode::Spectator,
        _ => {
            client.send_message(format!("{} is not a valid gamemode", gamemode).color(Color::RED));
            return Ok(());
        }
    };

    client.set_game_mode(new_gamemode);

    Ok(())
}

#[command]
pub fn setblock(
    client: Client<Game>,
    world: World<Game>,
    x: i64,
    y: i64,
    z: i64,
    block_type: String,
) -> Result<(), Box<dyn std::error::Error>> {
    dbg!(&x, &y, &z, &block_type);

    let chunk_pos = ChunkPos::at(x as f64, z as f64);

    if let Some(chunk) = world.chunks.get_mut(chunk_pos) {
        let block_x = x.rem_euclid(16);
        let block_z = z.rem_euclid(16);
        let block_y = y + 64;

        let block_kind = match BlockKind::from_str(&block_type) {
            Some(block_kind) => block_kind,
            None => {
                client
                    .send_message(format!("{} is not a valid block", block_type).color(Color::RED));
                return Ok(());
            }
        };

        let block_state = BlockState::from_kind(block_kind);
        println!("{} {} {}", block_x, block_y, block_z);

        let previous = chunk.set_block_state(
            block_x as usize,
            block_y as usize,
            block_z as usize,
            block_state,
        );

        // if previous == grassblock,
        //     check if there is grass on top,
        //         if so, check if the block we replaced it with can grow grass,
        //             if not, remove grass

        // and there are probably a ton more blocks with "weird" behavior like this.
        // how do we properly handle/structurize this?

        // same with like chests, check if it's a double-chest, if so, remove the other half.

        dbg!(previous);
    }

    Ok(())
}
