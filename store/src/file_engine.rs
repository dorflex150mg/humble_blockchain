use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use crate::engine::{Engine, StoreError};

const FILENAME: &str = "chain.dat";

/// Engine type that stores data in a file called `chain.dat` at the current directory.
pub struct FileEngine {
    writer: BufWriter<File>,
    reader: BufReader<File>,
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

        let write_file = File::options()
            .create(true)
            .write(true)
            .open(path.clone())
            .unwrap();

        let read_file = File::options().create(true).read(true).open(path).unwrap();

        FileEngine {
            writer: BufWriter::new(write_file),
            reader: BufReader::new(read_file),
        }
    }
}

impl Engine for FileEngine {
    fn store(&mut self, payload: &str) -> Result<(), StoreError> {
        self.writer
            .write(payload.as_bytes())
            .map_err(|_| StoreError::StorageError)
            .and(Ok(()))
    }

    fn load(&mut self) -> Result<String, StoreError> {
        let mut buffer: Vec<u8> = vec![];
        let n_bytes = self
            .reader
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
