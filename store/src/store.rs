use crate::{
    engine::{Engine, StoreError},
    file_engine::FileEngine,
};

/// `[Store]` reads and writes the Chain with some engine type. If none is provided, it uses the
/// default `[FileEngine]`.
pub struct Store {
    engine: Box<dyn Engine>,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            engine: Box::new(FileEngine::new()),
        }
    }
}

impl Store {
    /// Creates a new `Store` with the `engine` parameter. If `engine` is `None`, then `FileEngine`
    /// is utilized.
    #[must_use]
    pub fn new(engine: Option<Box<dyn Engine>>) -> Self {
        match engine {
            Some(e) => Store { engine: e },
            None => Store::default(),
        }
    }

    /// Uses the underlying `Engine` to store some `payload`.
    pub fn store(&mut self, payload: &str) -> Result<(), StoreError> {
        self.engine.store(payload)
    }

    /// Loads the contents kept on the underlying `StoreEngine`.
    pub fn load(&mut self) -> Result<String, StoreError> {
        self.engine.load()
    }
}
