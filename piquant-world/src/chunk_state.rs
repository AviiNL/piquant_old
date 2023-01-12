use std::time::Instant;

pub trait ChunkState {
    fn new(current_time: Instant, persistant: bool) -> Self
    where
        Self: Sized + Sync + Send;
    fn persistant(&self) -> bool;
    fn touch(&mut self, current_time: Instant);
    fn last_touched(&self) -> Instant;
}

// send and sync are required for the chunk state to be used in a par_iter_mut.
#[derive(Clone, Copy, Debug)]
pub struct DefaultChunkState {
    last_touched: Instant,
    persistant: bool,
}

impl Default for DefaultChunkState {
    fn default() -> Self {
        Self {
            last_touched: Instant::now(),
            persistant: false,
        }
    }
}

impl ChunkState for DefaultChunkState {
    fn new(current_time: Instant, persistant: bool) -> Self {
        Self {
            last_touched: current_time,
            persistant,
        }
    }

    fn persistant(&self) -> bool {
        self.persistant
    }

    fn touch(&mut self, current_time: Instant) {
        self.last_touched = current_time;
    }

    fn last_touched(&self) -> Instant {
        self.last_touched
    }
}

unsafe impl Send for DefaultChunkState {}
unsafe impl Sync for DefaultChunkState {}
