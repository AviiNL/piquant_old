use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;

use piquant_command::CommandService;
use piquant_world::{PiquantWorld, World, WorldState};

use valence::{
    prelude::{World as MCWorld, *},
    protocol::VarInt,
    server::{Server, SharedServer},
};

use crate::{client_state::ClientState, commands, server_state::ServerState};

pub struct Game {
    player_count: AtomicUsize,
    world: World<Game>,
    config: crate::config::Config,
    commands: CommandService<Game, Client<Game>, MCWorld<Game>>,
}

impl Game {
    pub fn new(config: crate::config::Config) -> Self {
        let world = World::new(
            config.world.seed.clone().into(),
            config.world.chunk_unload_delay,
        );

        let mut commands = CommandService::new();

        commands.add_command(commands::test_def(), commands::test);
        commands.add_command(commands::seed_def(), commands::seed);
        commands.add_command(commands::gamemode_def(), commands::gamemode);
        commands.add_command(commands::setblock_def(), commands::setblock);

        Self {
            player_count: AtomicUsize::new(0),
            world,
            config,
            commands,
        }
    }
}

#[async_trait]
impl Config for Game {
    type ServerState = ServerState;
    type ClientState = ClientState;
    type EntityState = ();
    type WorldState = piquant_world::PiquantWorld;
    type ChunkState = piquant_world::DefaultChunkState;
    type PlayerListState = ();
    type InventoryState = ();

    fn connection_mode(&self) -> ConnectionMode {
        ConnectionMode::Online
    }

    fn init(&self, server: &mut Server<Self>) {
        server.state.player_lists = Some(server.player_lists.insert(()).0);

        let world_folder = format!("worlds/{}", self.config.world.name);

        let mut world_state = PiquantWorld::new(world_folder);

        match world_state.read_all() {
            Ok(_) => {}
            Err(e) => {
                println!("Error reading region: {}", e);
                std::process::exit(1);
            }
        }

        world_state.read_spawnpoint().unwrap();

        let (_, world) = server.worlds.insert(DimensionId::default(), world_state);

        let mut player_spawn_point = Vec3::new(
            self.config.world.spawn.x as f64 + 0.5,
            0.0,
            self.config.world.spawn.z as f64 + 0.5,
        );

        // generate spawn area
        self.world.queue(
            world,
            player_spawn_point,
            self.config.world.view_distance,
            true,
        );

        self.world.update(world); // some kind of "progress" reporter would be nice

        if world.state.spawn.is_none() {
            // get spawn height
            player_spawn_point.y = match self.world.get_terrain_height(world, player_spawn_point) {
                Some(height) => height as f64 - 63.0,
                None => 0.0,
            };

            world.state.spawn = Some(player_spawn_point);
            world.state.seed = Some(self.world.seed());
        }

        dbg!(world.state.spawn);
    }

    async fn server_list_ping(
        &self,
        _server: &SharedServer<Self>,
        _remote_addr: SocketAddr,
        _protocol_version: i32,
    ) -> ServerListPing {
        ServerListPing::Respond {
            online_players: self.player_count.load(Ordering::SeqCst) as i32,
            max_players: self.config.network.max_players as i32,
            player_sample: Default::default(),
            description: self.config.network.description.clone().into(),
            favicon_png: None, //Some(include_bytes!("./assets/logo-64x64.png").as_slice().into()),
        }
    }

    fn address(&self) -> SocketAddr {
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), self.config.network.port).into()
    }

    fn update(&self, server: &mut Server<Self>) {
        let (world_id, world) = server.worlds.iter_mut().next().unwrap();

        server.clients.retain(|_id, client| {
            if client.created_this_tick() {
                if self
                    .player_count
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
                        (count < self.config.network.max_players).then_some(count + 1)
                    })
                    .is_err()
                {
                    client.disconnect("The server is full!".color(Color::RED));
                    return false;
                }

                if world.state.spawn.is_none() {
                    client.disconnect(
                        "Calm your tits, the server is still loading...".color(Color::RED),
                    );
                    return false;
                }

                match server
                    .entities
                    .insert_with_uuid(EntityKind::Player, client.uuid(), ())
                {
                    Some((id, entity)) => {
                        entity.set_world(world_id);
                        client.state.entity_id = id
                    }
                    None => {
                        client.disconnect(format!(
                            "Player with UUID {} already connected",
                            client.uuid()
                        ));
                        return false;
                    }
                }

                let spawn = world.state.spawn.as_ref().unwrap();

                client.respawn(world_id);
                client.set_flat(true);

                // client.queue_packet(&valence::protocol::packets::s2c::login::)

                let (root_id, commands) = self.commands.get_command_defs();
                client.queue_packet(&valence::protocol::packets::s2c::play::Commands {
                    commands,
                    root_index: VarInt(root_id),
                });

                // make a string slice from self.gamemode
                let gamemode = self.config.gameplay.gamemode.clone();
                client.set_game_mode(gamemode.into());

                client.teleport([spawn.x, spawn.y, spawn.z], 0.0, 0.0);
                client.set_player_list(server.state.player_lists.clone());

                // client.player_mut().

                if let Some(id) = &server.state.player_lists {
                    server.player_lists[id].insert(
                        client.uuid(),
                        client.username(),
                        client.textures().cloned(),
                        client.game_mode(),
                        0, // TODO: ping
                        None,
                        true,
                    );
                }
            }
            let player = &mut server.entities[client.state.entity_id];

            while let Some(event) = client.next_event() {
                match event {
                    ClientEvent::PlayerSession { .. } => {}
                    ClientEvent::ChatMessage { message, timestamp } => {
                        server.state.message_queue.queue_chat(
                            client.username(),
                            message.to_string(),
                            timestamp,
                        );
                    }
                    ClientEvent::ChatCommand { command, .. } => {
                        if let Err(e) = self.commands.execute(&command, self, client, world) {
                            client.send_message(format!("Error: {}", e).color(Color::RED));
                        }
                    }
                    _ => event.handle_default(client, player),
                }
            }

            let client_dist = client.view_distance();
            let server_dist = self.config.world.view_distance;

            let view_distance = std::cmp::min(client_dist, server_dist);

            let p = client.position();

            self.world.queue(world, p, view_distance, false);

            if client.is_disconnected() {
                println!("{} disconnected", client.username());
                self.player_count.fetch_sub(1, Ordering::SeqCst);
                if let Some(id) = &server.state.player_lists {
                    server.player_lists[id].remove(client.uuid());
                }
                player.set_deleted(true);

                return false;
            }

            true
        });

        while let Some(message) = server.state.message_queue.pop_front() {
            server.clients.iter_mut().for_each(|(_id, client)| {
                client.send_message(message.clone());
            });
        }

        self.world.update(world);
    }
}
