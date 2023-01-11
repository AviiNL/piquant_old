mod chunk_state;
mod seed;
mod world_state;

pub use self::seed::Seed;

use noise::{NoiseFn, SuperSimplex};
use rayon::prelude::ParallelIterator;
use valence::{prelude::*, protocol::BlockState};
use vek::Lerp;

pub use chunk_state::ChunkState;
pub use world_state::WorldState;

pub use chunk_state::DefaultChunkState;

pub struct WorldGen<G>
where
    G: Config,
{
    density_noise: SuperSimplex,
    hilly_noise: SuperSimplex,
    stone_noise: SuperSimplex,
    gravel_noise: SuperSimplex,
    grass_noise: SuperSimplex,
    _marker: std::marker::PhantomData<G>,
}

impl<G> WorldGen<G>
where
    G: Config,
    G::ChunkState: ChunkState + Send + Sync,
{
    pub fn new(seed: Seed) -> Self {
        let seed: u32 = seed.into();

        Self {
            density_noise: SuperSimplex::new(seed),
            hilly_noise: SuperSimplex::new(seed.wrapping_add(1)),
            stone_noise: SuperSimplex::new(seed.wrapping_add(2)),
            gravel_noise: SuperSimplex::new(seed.wrapping_add(3)),
            grass_noise: SuperSimplex::new(seed.wrapping_add(4)),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn queue(&self, world: &mut World<G>, position: Vec3<f64>, distance: u8, persistant: bool) {
        for pos in ChunkPos::at(position.x, position.z).in_view(distance) {
            if let Some(chunk) = world.chunks.get_mut(pos) {
                chunk.state.load();
            } else {
                world.chunks.insert(
                    pos,
                    UnloadedChunk::default(),
                    G::ChunkState::new(true, persistant),
                );
            }
        }
    }

    pub fn get_terrain_height(&self, world: &World<G>, position: Vec3<f64>) -> Option<i32> {
        let chunk_pos = ChunkPos::at(position.x, position.z);

        let chunk = match world.chunks.get(chunk_pos) {
            Some(chunk) => chunk,
            None => return None,
        };

        let block_in_chunk_x = (position.x % 16.0) as usize;
        let block_in_chunk_z = (position.z % 16.0) as usize;

        for y in (0..chunk.section_count() * 16).rev() {
            if chunk.block_state(block_in_chunk_x, y, block_in_chunk_z) != BlockState::AIR {
                return Some(y as i32);
            }
        }

        None
    }

    pub fn update(&self, world: &mut World<G>) {
        // Remove chunks outside the view distance of players.
        for (_, chunk) in world.chunks.iter_mut() {
            if !chunk.state.persistant() {
                chunk.set_deleted(!chunk.state.keep_loaded());
                chunk.state.unload();
            }
        }

        // Generate chunk data for chunks created this tick.
        world.chunks.par_iter_mut().for_each(|(pos, chunk)| {
            if !chunk.created_this_tick() {
                return;
            }

            for z in 0..16 {
                for x in 0..16 {
                    let block_x = x as i64 + pos.x as i64 * 16;
                    let block_z = z as i64 + pos.z as i64 * 16;

                    let mut in_terrain = false;
                    let mut depth = 0;

                    for y in (0..chunk.section_count() * 16).rev() {
                        if y == 0 {
                            chunk.set_block_state(x, y, z, BlockState::BEDROCK);
                            continue;
                        }

                        let b = terrain_column(
                            self,
                            block_x,
                            y as i64,
                            block_z,
                            &mut in_terrain,
                            &mut depth,
                        );
                        chunk.set_block_state(x, y, z, b);
                    }

                    // Add grass
                    for y in (1..chunk.section_count() * 16).rev() {
                        if chunk.block_state(x, y, z).is_air()
                            && chunk.block_state(x, y - 1, z) == BlockState::GRASS_BLOCK
                        {
                            let density = fbm(
                                &self.grass_noise,
                                [block_x, y as i64, block_z].map(|a| a as f64 / 5.0),
                                4,
                                2.0,
                                0.7,
                            );

                            if density > 0.55 {
                                if density > 0.7 && chunk.block_state(x, y + 1, z).is_air() {
                                    let upper = BlockState::TALL_GRASS
                                        .set(PropName::Half, PropValue::Upper);
                                    let lower = BlockState::TALL_GRASS
                                        .set(PropName::Half, PropValue::Lower);

                                    chunk.set_block_state(x, y + 1, z, upper);
                                    chunk.set_block_state(x, y, z, lower);
                                } else {
                                    chunk.set_block_state(x, y, z, BlockState::GRASS);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

fn terrain_column<G: Config>(
    wg: &WorldGen<G>,
    x: i64,
    y: i64,
    z: i64,
    in_terrain: &mut bool,
    depth: &mut u32,
) -> BlockState {
    const WATER_HEIGHT: i64 = 55;

    if has_terrain_at(wg, x, y, z) {
        let gravel_height = WATER_HEIGHT
            - 1
            - (fbm(
                &wg.gravel_noise,
                [x, y, z].map(|a| a as f64 / 10.0),
                3,
                2.0,
                0.5,
            ) * 6.0)
                .floor() as i64;

        if *in_terrain {
            if *depth > 0 {
                *depth -= 1;
                if y < gravel_height {
                    BlockState::GRAVEL
                } else {
                    BlockState::DIRT
                }
            } else {
                BlockState::STONE
            }
        } else {
            *in_terrain = true;
            let n = noise01(&wg.stone_noise, [x, y, z].map(|a| a as f64 / 15.0));

            *depth = (n * 5.0).round() as u32;

            if y < gravel_height {
                BlockState::GRAVEL
            } else if y < WATER_HEIGHT - 1 {
                BlockState::DIRT
            } else {
                BlockState::GRASS_BLOCK
            }
        }
    } else {
        *in_terrain = false;
        *depth = 0;
        if y < WATER_HEIGHT {
            BlockState::WATER
        } else {
            BlockState::AIR
        }
    }
}

fn has_terrain_at<G: Config>(wg: &WorldGen<G>, x: i64, y: i64, z: i64) -> bool {
    let hilly = Lerp::lerp_unclamped(
        0.1,
        1.0,
        noise01(&wg.hilly_noise, [x, y, z].map(|a| a as f64 / 400.0)).powi(2),
    );

    let lower = 15.0 + 100.0 * hilly;
    let upper = lower + 100.0 * hilly;

    if y as f64 <= lower {
        return true;
    } else if y as f64 >= upper {
        return false;
    }

    let density = 1.0 - lerpstep(lower, upper, y as f64);

    let n = fbm(
        &wg.density_noise,
        [x, y, z].map(|a| a as f64 / 100.0),
        4,
        2.0,
        0.5,
    );
    n < density
}

fn lerpstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    if x <= edge0 {
        0.0
    } else if x >= edge1 {
        1.0
    } else {
        (x - edge0) / (edge1 - edge0)
    }
}

fn fbm(noise: &SuperSimplex, p: [f64; 3], octaves: u32, lacunarity: f64, persistence: f64) -> f64 {
    let mut freq = 1.0;
    let mut amp = 1.0;
    let mut amp_sum = 0.0;
    let mut sum = 0.0;

    for _ in 0..octaves {
        let n = noise01(noise, p.map(|a| a * freq));
        sum += n * amp;
        amp_sum += amp;

        freq *= lacunarity;
        amp *= persistence;
    }

    // Scale the output to [0, 1]
    sum / amp_sum
}

fn noise01(noise: &SuperSimplex, xyz: [f64; 3]) -> f64 {
    (noise.get(xyz) + 1.0) / 2.0
}
