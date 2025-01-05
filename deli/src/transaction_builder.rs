use crate::{database::Database, error::Error, model::Model, transaction::Transaction};

/// Builder for [`Transaction`]
#[derive(Debug)]
pub struct TransactionBuilder<'a> {
    database: &'a idb::Database,
    mode: idb::TransactionMode,
    stores: Vec<&'a str>,
}

impl<'a> TransactionBuilder<'a> {
    /// Creates a new [`TransactionBuilder`] with the given database.
    pub fn new(database: &'a Database) -> Self {
        Self {
            database: database.as_idb_database(),
            mode: idb::TransactionMode::ReadOnly,
            stores: Vec::new(),
        }
    }

    /// Enables write access to the transaction.
    pub fn writable(mut self) -> Self {
        self.mode = idb::TransactionMode::ReadWrite;
        self
    }

    /// Adds a model to transaction
    pub fn with_model<M>(mut self) -> Self
    where
        M: Model,
    {
        self.stores.push(M::NAME);
        self
    }

    /// Builds the transaction
    pub fn build(self) -> Result<Transaction, Error> {
        self.database
            .transaction(&self.stores, self.mode)
            .map(Transaction::new)
            .map_err(Into::into)
    }
}
