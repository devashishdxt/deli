use std::mem::take;

use idb::{Transaction as IdbTransaction, TransactionMode};

use crate::{Database, Error, Model, Store};

/// Indexed db transaction
pub struct Transaction {
    stores: Vec<&'static str>,
    write: bool,
    transaction: IdbTransaction,
}

impl Transaction {
    /// Returns a builder for transaction
    pub fn builder(database: &Database) -> TransactionBuilder<'_> {
        TransactionBuilder::new(database)
    }

    /// Commits the transaction
    pub async fn commit(self) -> Result<(), Error> {
        self.transaction.commit().await.map_err(Into::into)
    }

    /// Waits for transaction to finish
    pub async fn done(self) -> Result<(), Error> {
        self.transaction.abort().await.map_err(Into::into)
    }

    /// Aborts the transaction
    pub async fn abort(self) -> Result<(), Error> {
        self.transaction.abort().await.map_err(Into::into)
    }

    /// Returns the stores in transaction
    pub fn store_names(&self) -> &[&'static str] {
        &self.stores
    }

    /// Returns true if the current transaction has write mode enabled
    pub fn is_write(&self) -> bool {
        self.write
    }

    /// Returns a store for a model
    pub fn store<M>(&self) -> Result<Store<M>, Error>
    where
        M: Model,
    {
        self.transaction
            .object_store(M::NAME)
            .map(Store::new)
            .map_err(Into::into)
    }
}

/// Builder for indexed db transactions
#[derive(Debug)]
pub struct TransactionBuilder<'a> {
    database: &'a Database,
    write: bool,
    stores: Vec<&'static str>,
}

impl<'a> TransactionBuilder<'a> {
    /// Creates a new transaction builder
    pub fn new(database: &'a Database) -> Self {
        Self {
            database,
            write: false,
            stores: Default::default(),
        }
    }

    /// Enables write mode on transaction
    pub fn writable(&mut self) -> &mut Self {
        self.write = true;
        self
    }

    /// Adds a model to transaction
    pub fn with_model<M>(&mut self) -> &mut Self
    where
        M: Model,
    {
        self.stores.push(M::NAME);
        self
    }

    /// Builds the transaction
    pub fn build(&mut self) -> Result<Transaction, Error> {
        let database = self.database.database();

        let mode = if self.write {
            TransactionMode::ReadWrite
        } else {
            TransactionMode::ReadOnly
        };

        let transaction = database.transaction(&self.stores, mode)?;

        Ok(Transaction {
            stores: take(&mut self.stores),
            write: self.write,
            transaction,
        })
    }
}
