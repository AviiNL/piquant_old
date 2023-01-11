use vek::Vec3;

pub struct WorldState {
    pub spawn: Option<Vec3<f64>>,
}

impl WorldState {
    pub fn new() -> Self {
        Self { spawn: None }
    }
}
