use crate::{
    database_builder::DatabaseBuilder, error::Error, transaction_builder::TransactionBuilder,
};

/// Provides connection to an indexed db database
#[derive(Debug)]
pub struct Database {
    database: idb::Database,
}

impl Database {
    pub(crate) fn new(database: idb::Database) -> Self {
        Self { database }
    }

    /// Returns a builder for [`Database`]
    pub fn builder(name: &str) -> DatabaseBuilder {
        DatabaseBuilder::new(name)
    }

    /// Returns the name of database
    pub fn name(&self) -> String {
        self.database.name()
    }

    /// Returns the version of database
    pub fn version(&self) -> Result<u32, Error> {
        self.database.version().map_err(Into::into)
    }

    /// Returns a transaction builder for creating transactions on database
    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self)
    }

    /// Closes database connection
    pub fn close(&self) {
        self.database.close();
    }

    /// Deletes a database
    pub async fn delete(name: &str) -> Result<(), Error> {
        idb::Factory::new()?.delete(name)?.await.map_err(Into::into)
    }

    pub(crate) fn as_idb_database(&self) -> &idb::Database {
        &self.database
    }
}
