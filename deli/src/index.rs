use std::{borrow::Borrow, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::Serializer;

use crate::{Cursor, Direction, Error, KeyCursor, KeyRange, Model, Transaction};

/// An index in indexed db object store
#[derive(Debug)]
pub struct Index<'t, M, T>
where
    M: Model,
    T: Serialize + DeserializeOwned,
{
    index: idb::Index,
    transaction: &'t Transaction,
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
            transaction,
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
            .count(query.into().try_into()?)?
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
        let js_value = self.index.get(key)?.await?;
        js_value
            .and_then(|js_value| {
                serde_wasm_bindgen::from_value(js_value)
                    .map_err(Into::into)
                    .transpose()
            })
            .transpose()
    }

    /// Gets the primary key corresponding to specified index key
    pub async fn get_key<K>(&self, key: &K) -> Result<Option<M::Key>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = key.serialize(&Serializer::json_compatible())?;
        let js_value = self.index.get_key(key)?.await?;
        js_value
            .and_then(|js_value| {
                serde_wasm_bindgen::from_value(js_value)
                    .map_err(Into::into)
                    .transpose()
            })
            .transpose()
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
        let js_values = self.index.get_all(query.into().try_into()?, limit)?.await?;

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
            .get_all_keys(query.into().try_into()?, limit)?
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
        let Some(mut cursor) = self.cursor(query, direction).await? else {
            return Ok(Vec::new());
        };

        if let Some(offset) = offset {
            if let Some(new_cursor) = cursor.advance(offset).await? {
                *cursor = new_cursor;
            } else {
                return Ok(Vec::new());
            }
        }

        match limit {
            Some(limit) => {
                let mut values = Vec::new();

                for _ in 0..limit {
                    match cursor.get_value()? {
                        Some(value) => {
                            values.push(value);
                            if let Some(new_cursor) = cursor.advance(1).await? {
                                *cursor = new_cursor;
                            } else {
                                break;
                            }
                        }
                        None => break,
                    }
                }

                Ok(values)
            }
            None => {
                let mut values = Vec::new();

                while let Some(value) = cursor.get_value()? {
                    values.push(value);
                    if let Some(new_cursor) = cursor.advance(1).await? {
                        *cursor = new_cursor;
                    } else {
                        break;
                    }
                }

                Ok(values)
            }
        }
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
        let Some(mut cursor) = self.key_cursor(query, direction).await? else {
            return Ok(Vec::new());
        };

        if let Some(offset) = offset {
            if let Some(new_cursor) = cursor.advance(offset).await? {
                *cursor = new_cursor;
            } else {
                return Ok(Vec::new());
            }
        }

        match limit {
            Some(limit) => {
                let mut keys = Vec::new();

                for _ in 0..limit {
                    match cursor.get_key()? {
                        Some(value) => {
                            keys.push(value);
                            if let Some(new_cursor) = cursor.advance(1).await? {
                                *cursor = new_cursor;
                            } else {
                                break;
                            }
                        }
                        None => break,
                    }
                }

                Ok(keys)
            }
            None => {
                let mut keys = Vec::new();

                while let Some(value) = cursor.get_key()? {
                    keys.push(value);
                    if let Some(new_cursor) = cursor.advance(1).await? {
                        *cursor = new_cursor;
                    } else {
                        break;
                    }
                }

                Ok(keys)
            }
        }
    }

    /// Returns a cursor on index
    pub async fn cursor<'a, K>(
        &self,
        query: impl Into<KeyRange<'a, M, T, K>>,
        direction: Option<Direction>,
    ) -> Result<Option<M::Cursor<'t>>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized + 'a,
    {
        let cursor = self
            .index
            .open_cursor(query.into().try_into()?, direction)?
            .await?;

        Ok(cursor.map(|c| Cursor::new(self.transaction, c).into()))
    }

    /// Returns a key cursor on index
    pub async fn key_cursor<'a, K>(
        &self,
        query: impl Into<KeyRange<'a, M, T, K>>,
        direction: Option<Direction>,
    ) -> Result<Option<M::KeyCursor<'t>>, Error>
    where
        T: Borrow<K>,
        K: Serialize + ?Sized + 'a,
    {
        let cursor = self
            .index
            .open_key_cursor(query.into().try_into()?, direction)?
            .await?;

        Ok(cursor.map(|c| KeyCursor::new(self.transaction, c).into()))
    }
}
