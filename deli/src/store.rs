use std::{
    borrow::Borrow,
    convert::TryInto,
    marker::PhantomData,
    ops::{Range, RangeInclusive},
};

use idb::{ObjectStore, Query};
use serde::Serialize;
use serde_wasm_bindgen::Serializer;

use crate::{Direction, Error, Model};

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
        query: Option<KeyRange<'a, M, K>>,
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
        query: Option<KeyRange<'a, M, K>>,
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
        query: Option<KeyRange<'a, M, K>>,
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
        query: Option<KeyRange<'a, M, K>>,
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

    /// Deletes value with specified key
    pub async fn delete<'a, K>(&self, query: KeyRange<'a, M, K>) -> Result<(), Error>
    where
        M::Key: Borrow<K>,
        K: Serialize + ?Sized,
    {
        let query: Query = query.try_into()?;
        self.store.delete(query).await.map_err(Into::into)
    }
}

/// Defines the range of keys
pub struct KeyRange<'a, M, K>
where
    M: Model,
    M::Key: Borrow<K>,
    K: Serialize + ?Sized,
{
    inner: KeyRangeInner<'a, K>,
    _generis: PhantomData<M>,
}

enum KeyRangeInner<'a, K>
where
    K: Serialize + ?Sized,
{
    Singe(&'a K),
    Range(Range<&'a K>),
    RangeInclusive(RangeInclusive<&'a K>),
}

impl<'a, M, K> From<&'a K> for KeyRange<'a, M, K>
where
    M: Model,
    M::Key: Borrow<K>,
    K: Serialize + ?Sized,
{
    fn from(single: &'a K) -> Self {
        Self {
            inner: KeyRangeInner::Singe(single),
            _generis: Default::default(),
        }
    }
}

impl<'a, M, K> From<Range<&'a K>> for KeyRange<'a, M, K>
where
    M: Model,
    M::Key: Borrow<K>,
    K: Serialize + ?Sized,
{
    fn from(range: Range<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::Range(range),
            _generis: Default::default(),
        }
    }
}

impl<'a, M, K> From<RangeInclusive<&'a K>> for KeyRange<'a, M, K>
where
    M: Model,
    M::Key: Borrow<K>,
    K: Serialize + ?Sized,
{
    fn from(range: RangeInclusive<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::RangeInclusive(range),
            _generis: Default::default(),
        }
    }
}

impl<'a, M, K> TryFrom<KeyRange<'a, M, K>> for Query
where
    M: Model,
    M::Key: Borrow<K>,
    K: Serialize + ?Sized,
{
    type Error = Error;

    fn try_from(key_range: KeyRange<'a, M, K>) -> Result<Self, Self::Error> {
        match key_range.inner {
            KeyRangeInner::Singe(singe) => {
                let js_value = singe.serialize(&Serializer::json_compatible())?;
                Ok(Query::Key(js_value))
            }
            KeyRangeInner::Range(range) => {
                let lower = range.start.serialize(&Serializer::json_compatible())?;
                let upper = range.end.serialize(&Serializer::json_compatible())?;

                let key_range = idb::KeyRange::bound(&lower, &upper, Some(false), Some(true))?;

                Ok(Query::KeyRange(key_range))
            }
            KeyRangeInner::RangeInclusive(range) => {
                let lower = range.start().serialize(&Serializer::json_compatible())?;
                let upper = range.end().serialize(&Serializer::json_compatible())?;

                let key_range = idb::KeyRange::bound(&lower, &upper, Some(false), Some(false))?;

                Ok(Query::KeyRange(key_range))
            }
        }
    }
}
