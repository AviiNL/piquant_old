use vek::Vec3;

use crate::Seed;

pub struct WorldState {
    pub spawn: Option<Vec3<f64>>,
    pub seed: Option<Seed>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            spawn: None,
            seed: None,
        }
    }
}
