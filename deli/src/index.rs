use std::{borrow::Borrow, marker::PhantomData};

use idb::Query;
use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::Serializer;

use crate::{Direction, Error, KeyRange, Model};

/// An index in indexed db object store
pub struct Index<M, T>
where
    M: Model,
    T: DeserializeOwned,
{
    index: idb::Index,
    _generics_model: PhantomData<M>,
    _generics_index_type: PhantomData<T>,
}

impl<M, T> Index<M, T>
where
    M: Model,
    T: Serialize + DeserializeOwned,
{
    /// Creates a new instance of index
    pub(crate) fn new(index: idb::Index) -> Self {
        Self {
            index,
            _generics_model: Default::default(),
            _generics_index_type: Default::default(),
        }
    }

    /// Counts all the values from store with given query and limit
    pub async fn count<'a, K>(&self, query: Option<KeyRange<'a, M, T, K>>) -> Result<u32, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query = query.map(TryInto::try_into).transpose()?;
        self.index.count(query).await.map_err(Into::into)
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

    /// Gets all the values from store with given query and limit
    pub async fn get_all<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, T, K>>,
        limit: Option<u32>,
    ) -> Result<Vec<M>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query = query.map(TryInto::try_into).transpose()?;
        let js_values = self.index.get_all(query, limit).await?;

        js_values
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .collect()
    }

    /// Gets all the keys from store with given query and limit
    pub async fn get_all_keys<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, T, K>>,
        limit: Option<u32>,
    ) -> Result<Vec<T>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query = query.map(TryInto::try_into).transpose()?;
        let js_keys = self.index.get_all_keys(query, limit).await?;

        js_keys
            .into_iter()
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .collect()
    }

    /// Scans the store for values
    pub async fn scan<'a, K>(
        &self,
        query: Option<KeyRange<'a, M, T, K>>,
        direction: Option<Direction>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<M>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query: Option<Query> = query.map(TryInto::try_into).transpose()?;
        let mut cursor = self.index.open_cursor(query, direction).await?;

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
        query: Option<KeyRange<'a, M, T, K>>,
        direction: Option<Direction>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<T>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query: Option<Query> = query.map(TryInto::try_into).transpose()?;
        let mut cursor = self.index.open_cursor(query, direction).await?;

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
}
