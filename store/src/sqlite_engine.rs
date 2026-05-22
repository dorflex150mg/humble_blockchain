use crate::engine::{Engine, StoreError};
use sqlite::{Connection, Statement, open};

pub struct SQLiteEngine {
    connection: Connection,
}

const FILENAME: &str = "chain.sql";
const TABLE: &str = "Chain";

impl SQLiteEngine {
    pub fn try_new(path: Option<&str>) -> Result<Self, sqlite::Error> {
        let connection = sqlite::open(path.unwrap_or(FILENAME))?;
        Ok(SQLiteEngine { connection })
    }
}

impl Engine for SQLiteEngine {
    fn store(&mut self, payload: &str) -> Result<(), StoreError> {
        self.connection
            .execute("CREATE IF NOT EXISTS TABLE Chain(TEXT chain_content);")
            .map_err(|_| StoreError::SetupError)?;
        let query = "UPDATE Chain SET chain_content = ?";
        let mut statement = self
            .connection
            .prepare(query)
            .map_err(|_| StoreError::PrepareError)?;
        statement
            .bind((1, payload))
            .map_err(|_| StoreError::PrepareError)?;
        statement.next().map_err(|_| StoreError::StorageError)?;
        Ok(())
    }

    fn load(&mut self) -> Result<String, StoreError> {
        todo!()
    }
}
