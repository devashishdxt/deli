use std::ops::Deref;

use idb::builder::ObjectStoreBuilder;
use serde::{de::DeserializeOwned, Serialize};

use crate::{error::Error, object_store::ObjectStore, transaction::Transaction};

/// Trait for defining object stores in an indexed db database
pub trait Model: Serialize + DeserializeOwned {
    /// Name of the object store
    const NAME: &'static str;

    /// Type of key for the model
    type Key: Serialize + DeserializeOwned;

    /// Type of value for the model (used to insert operations)
    type Add: Serialize;

    /// Type of object store for the model
    type ObjectStore<'t>: Deref<Target = ObjectStore<'t, Self>> + From<ObjectStore<'t, Self>>;

    /// Get a store from given transaction
    fn with_transaction(transaction: &Transaction) -> Result<Self::ObjectStore<'_>, Error> {
        transaction.object_store::<Self>().map(Into::into)
    }

    /// Returns the object store builder for the model
    #[doc(hidden)]
    fn object_store_builder() -> ObjectStoreBuilder;
}
