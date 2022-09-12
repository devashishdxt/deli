use std::{borrow::Borrow, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::Serializer;

use crate::{Direction, Error, KeyRange, Model, Transaction};

/// An index in indexed db object store
#[derive(Debug)]
pub struct Index<'t, M, T>
where
    M: Model,
    T: DeserializeOwned,
{
    index: idb::Index,
    _transaction: &'t Transaction,
    _generics_model: PhantomData<M>,
    _generics_index_type: PhantomData<T>,
}

impl<'t, M, T> Index<'t, M, T>
where
    M: Model,
    T: Serialize + DeserializeOwned,
{
    /// Creates a new instance of index
    pub(crate) fn new(transaction: &'t Transaction, index: idb::Index) -> Self {
        Self {
            index,
            _transaction: transaction,
            _generics_model: Default::default(),
            _generics_index_type: Default::default(),
        }
    }

    /// Counts all the values from store with given query and limit
    pub async fn count<'a, K>(&self, query: impl Into<KeyRange<'a, M, T, K>>) -> Result<u32, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized + 'a,
    {
        self.index
            .count(query.into().try_into()?)
            .await
            .map_err(Into::into)
    }

    /// Gets value for specifier key
    pub async fn get<K>(&self, key: &K) -> Result<Option<M>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = key.serialize(&Serializer::json_compatible())?;
        let js_value = self.index.get(key).await?;
        serde_wasm_bindgen::from_value(js_value).map_err(Into::into)
    }

    /// Gets the primary key corresponding to specified index key
    pub async fn get_key<K>(&self, key: &K) -> Result<Option<M::Key>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = key.serialize(&Serializer::json_compatible())?;
        let js_value = self.index.get_key(key).await?;
        serde_wasm_bindgen::from_value(js_value).map_err(Into::into)
    }

    /// Gets all the values from store with given query and limit
    pub async fn get_all<'a, K>(
        &self,
        query: impl Into<KeyRange<'a, M, T, K>>,
        limit: Option<u32>,
    ) -> Result<Vec<M>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized + 'a,
    {
        let js_values = self.index.get_all(query.into().try_into()?, limit).await?;

        js_values
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .collect()
    }

    /// Gets all the keys from store with given query and limit
    pub async fn get_all_keys<'a, K>(
        &self,
        query: impl Into<KeyRange<'a, M, T, K>>,
        limit: Option<u32>,
    ) -> Result<Vec<M::Key>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized + 'a,
    {
        let js_keys = self
            .index
            .get_all_keys(query.into().try_into()?, limit)
            .await?;

        js_keys
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .collect()
    }

    /// Scans the store for values
    pub async fn scan<'a, K>(
        &self,
        query: impl Into<KeyRange<'a, M, T, K>>,
        direction: Option<Direction>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<M>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized + 'a,
    {
        let mut cursor = self
            .index
            .open_cursor(query.into().try_into()?, direction)
            .await?;

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
        query: impl Into<KeyRange<'a, M, T, K>>,
        direction: Option<Direction>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<M::Key>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized + 'a,
    {
        let mut cursor = self
            .index
            .open_cursor(query.into().try_into()?, direction)
            .await?;

        if let Some(offset) = offset {
            cursor.advance(offset).await?;
        }

        let js_keys = match limit {
            Some(limit) => {
                let mut js_keys = Vec::new();

                for _ in 0..limit {
                    let js_key = cursor.primary_key()?;

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
                    let js_key = cursor.primary_key()?;

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
}
