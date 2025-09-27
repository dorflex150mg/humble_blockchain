use std::{
    fs::File,
    io::{Read, Seek, Write},
};

use crate::engine::{Engine, StoreError};

const FILENAME: &str = "chain.dat";

/// Engine type that stores data in a file called `chain.dat` at the current directory.
pub struct FileEngine {
    file: File,
}

impl Default for FileEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FileEngine {
    /// Creates a new `[FileEngine]` instance.
    #[allow(clippy::unwrap_used)]
    pub fn new() -> Self {
        let mut path = std::env::current_dir().unwrap();
        path.push(FILENAME);
        println!("{}", path.display());

        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.clone())
            .unwrap();

        FileEngine { file }
    }
}

impl Engine for FileEngine {
    fn store(&mut self, payload: &str) -> Result<(), StoreError> {
        let _ = self.file.set_len(0);
        let res = self
            .file
            .write(payload.as_bytes())
            .map_err(|_| StoreError::StorageError)
            .and(Ok(()));
        let _ = self.file.flush();
        res
    }

    fn load(&mut self) -> Result<String, StoreError> {
        let _ = self.file.seek(std::io::SeekFrom::Start(0));
        let mut buffer: Vec<u8> = vec![];
        let n_bytes = self
            .file
            .read_to_end(&mut buffer)
            .map_err(|_| StoreError::LoadError)?;
        if n_bytes == 0 {
            return Err(StoreError::EmptyFile);
        }
        Ok(str::from_utf8(&buffer)
            .map_err(|_| StoreError::LoadError)?
            .to_owned())
    }
}
