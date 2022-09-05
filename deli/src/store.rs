use std::marker::PhantomData;

use idb::ObjectStore;
use serde::Serialize;
use serde_wasm_bindgen::Serializer;

use crate::{Error, Model};

pub struct Store<M>
where
    M: Model,
{
    store: ObjectStore,
    _generis: PhantomData<M>,
}

impl<M> Store<M>
where
    M: Model,
{
    /// Creates a new instance of store
    pub(crate) fn new(store: ObjectStore) -> Self {
        Self {
            store,
            _generis: Default::default(),
        }
    }

    /// Gets value for specifier key
    pub async fn get(&self, key: &M::Key) -> Result<Option<M>, Error> {
        let key = key.serialize(&Serializer::json_compatible())?;
        let js_value = self.store.get(key).await?;
        serde_wasm_bindgen::from_value(js_value).map_err(Into::into)
    }

    /// Adds a value to the store returning its key
    pub async fn add<V>(&self, value: &V) -> Result<M::Key, Error>
    where
        V: Serialize,
    {
        let value = value.serialize(&Serializer::json_compatible())?;
        let js_key = self.store.add(&value, None).await?;
        serde_wasm_bindgen::from_value(js_key).map_err(Into::into)
    }

    /// Updates a value in the store returning its key
    pub async fn update<V>(self, value: &V) -> Result<M::Key, Error>
    where
        V: Serialize,
    {
        let value = value.serialize(&Serializer::json_compatible())?;
        let js_key = self.store.put(&value, None).await?;
        serde_wasm_bindgen::from_value(js_key).map_err(Into::into)
    }

    /// Deletes value with specified key
    pub async fn delete<K>(&self, key: &K) -> Result<(), Error>
    where
        K: Serialize,
    {
        let key = key.serialize(&Serializer::json_compatible())?;
        self.store.delete(key).await.map_err(Into::into)
    }
}
