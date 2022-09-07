use std::{borrow::Borrow, convert::TryInto, marker::PhantomData};

use idb::{ObjectStore, Query};
use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::Serializer;

use crate::{Direction, Error, Index, KeyRange, Model};

/// An object store in indexed db (with add and update function)
pub struct NonGenericStore<'a, M>
where
    M: Model,
{
    store: &'a ObjectStore,
    _generics: PhantomData<M>,
}

impl<'a, M> NonGenericStore<'a, M>
where
    M: Model,
{
    /// Creates a new instance of store
    pub(crate) fn new(store: &'a ObjectStore) -> Self {
        Self {
            store,
            _generics: Default::default(),
        }
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
    pub async fn update<V>(&self, value: &V) -> Result<M::Key, Error>
    where
        V: Serialize,
    {
        let value = value.serialize(&Serializer::json_compatible())?;
        let js_key = self.store.put(&value, None).await?;
        serde_wasm_bindgen::from_value(js_key).map_err(Into::into)
    }

    /// Returns object store index with given name and data type
    pub fn index<T>(&self, name: &str) -> Result<Index<M, T>, Error>
    where
        T: Serialize + DeserializeOwned,
    {
        let index = self.store.index(name)?;
        Ok(Index::new(index))
    }
}

/// An object store in indexed db
pub struct Store<M>
where
    M: Model,
{
    store: ObjectStore,
    _generics: PhantomData<M>,
}

impl<M> Store<M>
where
    M: Model,
{
    /// Creates a new instance of store
    pub(crate) fn new(store: ObjectStore) -> Self {
        Self {
            store,
            _generics: Default::default(),
        }
    }

    /// Returns an add and update enabled store
    #[doc(hidden)]
    pub fn non_generic_store(&self) -> NonGenericStore<'_, M> {
        NonGenericStore::new(&self.store)
    }

    /// Counts all the values from store with given query and limit
    pub async fn count<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, M::Key, K>>,
    ) -> Result<u32, Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query = query.map(TryInto::try_into).transpose()?;
        self.store.count(query).await.map_err(Into::into)
    }

    /// Gets value for specifier key
    pub async fn get<K>(&self, key: &K) -> Result<Option<M>, Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = key.serialize(&Serializer::json_compatible())?;
        let js_value = self.store.get(key).await?;
        serde_wasm_bindgen::from_value(js_value).map_err(Into::into)
    }

    /// Gets all the values from store with given query and limit
    pub async fn get_all<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, M::Key, K>>,
        limit: Option<u32>,
    ) -> Result<Vec<M>, Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query = query.map(TryInto::try_into).transpose()?;
        let js_values = self.store.get_all(query, limit).await?;

        js_values
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .collect()
    }

    /// Gets all the keys from store with given query and limit
    pub async fn get_all_keys<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, M::Key, K>>,
        limit: Option<u32>,
    ) -> Result<Vec<M::Key>, Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query = query.map(TryInto::try_into).transpose()?;
        let js_keys = self.store.get_all_keys(query, limit).await?;

        js_keys
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .collect()
    }

    /// Scans the store for values
    pub async fn scan<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, M::Key, K>>,
        direction: Option<Direction>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<M>, Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query: Option<Query> = query.map(TryInto::try_into).transpose()?;
        let mut cursor = self.store.open_cursor(query, direction).await?;

        if let Some(offset) = offset {
            cursor.advance(offset).await?;
        }

        let js_values = match limit {
            Some(limit) => {
                let mut js_values = Vec::new();

                for _ in 0..limit {
                    let js_value = cursor.value()?;

                    if js_value.is_null() {
                        break;
                    }

                    js_values.push(js_value);
                    cursor.next(None).await?;
                }

                js_values
            }
            None => {
                let mut js_values = Vec::new();

                loop {
                    let js_value = cursor.value()?;

                    if js_value.is_null() {
                        break;
                    }

                    js_values.push(js_value);
                    cursor.next(None).await?;
                }

                js_values
            }
        };

        js_values
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .collect()
    }

    /// Scans the store for keys
    pub async fn scan_keys<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, M::Key, K>>,
        direction: Option<Direction>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<M::Key>, Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query: Option<Query> = query.map(TryInto::try_into).transpose()?;
        let mut cursor = self.store.open_cursor(query, direction).await?;

        if let Some(offset) = offset {
            cursor.advance(offset).await?;
        }

        let js_keys = match limit {
            Some(limit) => {
                let mut js_keys = Vec::new();

                for _ in 0..limit {
                    let js_key = cursor.key()?;

                    if js_key.is_null() {
                        break;
                    }

                    js_keys.push(js_key);
                    cursor.next(None).await?;
                }

                js_keys
            }
            None => {
                let mut js_keys = Vec::new();

                loop {
                    let js_key = cursor.key()?;

                    if js_key.is_null() {
                        break;
                    }

                    js_keys.push(js_key);
                    cursor.next(None).await?;
                }

                js_keys
            }
        };

        js_keys
            .into_iter()
            .map(|js_key| serde_wasm_bindgen::from_value(js_key).map_err(Into::into))
            .collect()
    }

    /// Deletes value with specified key
    pub async fn delete<'a, K>(&self, query: KeyRange<'a, M, M::Key, K>) -> Result<(), Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query: Query = query.try_into()?;
        self.store.delete(query).await.map_err(Into::into)
    }
}
