use std::ops::{Deref, DerefMut};

use idb::VersionChangeEvent;
use serde::{de::DeserializeOwned, Serialize};

use crate::{Cursor, Error, KeyCursor, Store, Transaction};

/// Trait for defining object stores in an indexed db database
pub trait Model: DeserializeOwned {
    /// Name of the object store
    const NAME: &'static str;

    /// Type of key for the model
    type Key: Serialize + DeserializeOwned;

    /// Type of store for the model
    type Store<'t>: Deref<Target = Store<'t, Self>> + From<Store<'t, Self>>;

    /// Type of value cursor for the model
    type Cursor<'t>: Deref<Target = Cursor<'t, Self>> + DerefMut + From<Cursor<'t, Self>>;

    /// Type of key cursor for the model
    type KeyCursor<'t>: Deref<Target = KeyCursor<'t, Self>> + DerefMut + From<KeyCursor<'t, Self>>;

    /// Upgrade handler for the object store
    #[doc(hidden)]
    fn handle_upgrade(event: VersionChangeEvent);

    /// Get a store from given transaction
    fn with_transaction(transaction: &Transaction) -> Result<Self::Store<'_>, Error> {
        transaction.store::<Self>().map(::core::convert::Into::into)
    }
}
