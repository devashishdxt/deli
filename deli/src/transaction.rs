use idb::{TransactionMode, TransactionResult};

use crate::{
    database::Database, error::Error, model::Model, object_store::ObjectStore,
    transaction_builder::TransactionBuilder,
};

/// Provides a transaction on a database. All reading and writing of data is done within transactions.
#[derive(Debug)]
pub struct Transaction {
    transaction: idb::Transaction,
}

impl Transaction {
    pub(crate) fn new(transaction: idb::Transaction) -> Self {
        Self { transaction }
    }

    /// Creates a new [`TransactionBuilder`] with the given database.
    pub fn builder(database: &Database) -> TransactionBuilder<'_> {
        TransactionBuilder::new(database)
    }

    /// Returns a list of the names of object stores in the transaction’s scope. For an upgrade transaction this is all
    /// object stores in the database.
    pub fn store_names(&self) -> Vec<String> {
        self.transaction.store_names()
    }

    /// Returns the mode the transaction was created with ("readonly" or "readwrite"), or "versionchange" for an upgrade
    /// transaction.
    pub fn mode(&self) -> Result<TransactionMode, Error> {
        self.transaction.mode().map_err(Into::into)
    }

    /// Returns an [`ObjectStore`] for a model in transaction's scope.
    pub fn object_store<M>(&self) -> Result<ObjectStore<'_, M>, Error>
    where
        M: Model,
    {
        self.transaction
            .object_store(M::NAME)
            .map(|object_store| ObjectStore::new(object_store, self))
            .map_err(Into::into)
    }

    /// Attempts to commit the transaction. All pending requests will be allowed to complete, but no new requests will
    /// be accepted. This can be used to force a transaction to quickly finish, without waiting for pending requests to
    /// fire success events before attempting to commit normally.
    pub async fn commit(self) -> Result<TransactionResult, Error> {
        self.transaction.commit()?.await.map_err(Into::into)
    }

    /// Aborts the transaction. All pending requests will fail and all changes made to the database will be reverted.
    pub async fn abort(self) -> Result<TransactionResult, Error> {
        self.transaction.abort()?.await.map_err(Into::into)
    }

    /// Waits for the transaction to complete and returns the result.
    pub async fn done(self) -> Result<TransactionResult, Error> {
        self.transaction.await.map_err(Into::into)
    }
}
