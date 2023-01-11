pub trait ChunkState {
    fn new(keep_loaded: bool, persistant: bool) -> Self
    where
        Self: Sized + Sync + Send;
    fn keep_loaded(&self) -> bool;
    fn persistant(&self) -> bool;
    fn load(&mut self);
    fn unload(&mut self);
}

// send and sync are required for the chunk state to be used in a par_iter_mut.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultChunkState {
    keep_loaded: bool,
    persistant: bool,
}

impl ChunkState for DefaultChunkState {
    fn new(keep_loaded: bool, persistant: bool) -> Self {
        Self {
            keep_loaded,
            persistant,
        }
    }

    fn keep_loaded(&self) -> bool {
        self.keep_loaded
    }

    fn persistant(&self) -> bool {
        self.persistant
    }

    fn load(&mut self) {
        self.keep_loaded = true;
    }

    fn unload(&mut self) {
        self.keep_loaded = false;
    }
}

unsafe impl Send for DefaultChunkState {}
unsafe impl Sync for DefaultChunkState {}
