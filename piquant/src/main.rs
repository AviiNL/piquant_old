mod chat;

mod client_state;
mod commands;
mod config;
mod server;
mod server_state;
use config::Config;
use server::Game;
use server_state::ServerState;

#[derive(Debug)]
enum Error {
    GenericError(Box<dyn std::error::Error>),
    ShutdownResult(Box<dyn std::error::Error + Send + Sync>),
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::GenericError(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::ShutdownResult(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let settings = Config::load_or_create("server.toml")?;

    valence::start_server(Game::new(settings), ServerState::new())?;

    Ok(())
}
