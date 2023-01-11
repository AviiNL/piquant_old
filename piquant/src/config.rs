use std::io::Read;

use serde::{Deserialize, Serialize};

use piquant_world::Seed;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub network: Network,
    pub world: World,
    pub gameplay: Gameplay,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
    pub port: u16,
    pub max_players: usize,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldSpawn {
    pub x: i32,
    pub z: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct World {
    pub seed: Seed,
    pub view_distance: u8,
    pub spawn: WorldSpawn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Gameplay {
    pub gamemode: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: Network {
                port: 25565,
                max_players: 16,
                description: "§bHello Valence!".into(),
            },
            world: World {
                seed: Seed::FromString("".to_string()),
                view_distance: 8,
                spawn: WorldSpawn { x: 0, z: 0 },
            },
            gameplay: Gameplay {
                gamemode: "survival".into(),
            },
        }
    }
}

impl Config {
    pub fn load_or_create(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if std::path::Path::new(filename).exists() {
            Self::load(filename)
        } else {
            Self::create(filename)
        }
    }

    pub fn create(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let settings = Self::default();
        let toml = toml::to_string(&settings)?;
        std::fs::write(filename, toml)?;
        Ok(settings)
    }

    pub fn load(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(filename)?;
        let mut reader = std::io::BufReader::new(file);
        // read into string slice
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        // parse string slice
        let settings: Config = toml::from_str(&contents)?;

        Ok(settings)
    }
}