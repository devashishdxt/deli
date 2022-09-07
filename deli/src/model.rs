use std::ops::Deref;

use idb::VersionChangeEvent;
use serde::{de::DeserializeOwned, Serialize};

use crate::{Error, Store, Transaction};

/// Trait for defining object stores in an indexed db database
pub trait Model: DeserializeOwned {
    /// Name of the object store
    const NAME: &'static str;

    /// Type of key for the model
    type Key: Serialize + DeserializeOwned;

    /// Type of store for the model
    type Store: From<Store<Self>> + Deref<Target = Store<Self>>;

    /// Upgrade handler for the object store
    fn handle_upgrade(event: VersionChangeEvent);

    /// Returns the store for the model
    fn with_transaction(transaction: &Transaction) -> Result<Self::Store, Error> {
        transaction.store::<Self>().map(Into::into)
    }
}
