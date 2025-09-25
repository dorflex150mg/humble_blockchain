use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Failed to store string.")]
    StorageError,
    #[error("Failed to load string.")]
    LoadError,
    #[error("Attempted to load string from an empty file.")]
    EmptyFile,
}

pub trait Engine {
    fn store(&mut self, payload: &str) -> Result<(), StoreError>;
    fn load(&mut self) -> Result<String, StoreError>;
}
