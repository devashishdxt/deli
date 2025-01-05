use std::borrow::Borrow;

use serde::{de::DeserializeOwned, Serialize};

use crate::{error::Error, model::Model, transaction::Transaction, JSON_SERIALIZER};

/// Cursor on an object store or index
#[derive(Debug)]
pub struct KeyCursor<'t, M, K> {
    cursor: idb::ManagedKeyCursor,
    _transaction: &'t Transaction,
    _marker: std::marker::PhantomData<(M, K)>,
}

impl<'t, M, K> KeyCursor<'t, M, K>
where
    M: Model,
    K: Serialize + DeserializeOwned,
{
    pub(crate) fn new(cursor: idb::ManagedKeyCursor, transaction: &'t Transaction) -> Self {
        Self {
            cursor,
            _transaction: transaction,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the direction of the cursor
    pub fn direction(&self) -> Result<Option<idb::CursorDirection>, Error> {
        self.cursor.direction().map_err(Into::into)
    }

    /// Returns the key at the current position of the cursor
    pub fn key(&self) -> Result<Option<K>, Error> {
        let js_value = self.cursor.key()?;
        js_value
            .map(serde_wasm_bindgen::from_value)
            .transpose()
            .map_err(Into::into)
    }

    /// Returns the primary key at the current position of the cursor
    pub fn primary_key(&self) -> Result<Option<M::Key>, Error> {
        let js_value = self.cursor.primary_key()?;
        js_value
            .map(serde_wasm_bindgen::from_value)
            .transpose()
            .map_err(Into::into)
    }

    /// Advances the cursor through the next count records in range.
    pub async fn advance(&mut self, count: u32) -> Result<(), Error> {
        self.cursor.advance(count).await.map_err(Into::into)
    }

    /// Advances the cursor to the next record in range matching or after key (if provided).
    pub async fn next<Q>(&mut self, key: Option<&Q>) -> Result<(), Error>
    where
        K: Borrow<Q>,
        Q: Serialize,
    {
        let js_value = key.map(|key| key.serialize(&JSON_SERIALIZER)).transpose()?;
        self.cursor
            .next(js_value.as_ref())
            .await
            .map_err(Into::into)
    }

    /// Advances the cursor to the next record in range matching or after key and primary key. Returns an [`Error`] if
    /// the source is not an [`Index`](crate::Index).
    pub async fn next_primary_key<Q, R>(&mut self, key: &Q, primary_key: &R) -> Result<(), Error>
    where
        K: Borrow<Q>,
        Q: Serialize,
        M::Key: Borrow<R>,
        R: Serialize,
    {
        let js_key = key.serialize(&JSON_SERIALIZER)?;
        let js_primary_key = primary_key.serialize(&JSON_SERIALIZER)?;
        self.cursor
            .next_primary_key(&js_key, &js_primary_key)
            .await
            .map_err(Into::into)
    }

    /// Updates the value at the current position of the cursor
    pub async fn update<V>(&mut self, value: &V) -> Result<M, Error>
    where
        M: Borrow<V>,
        V: Serialize,
    {
        let js_value = value.serialize(&JSON_SERIALIZER)?;
        let updated_js_value = self.cursor.update(&js_value).await?;
        serde_wasm_bindgen::from_value(updated_js_value).map_err(Into::into)
    }

    /// Deletes the value at the current position of the cursor
    pub async fn delete(&mut self) -> Result<(), Error> {
        self.cursor.delete().await.map_err(Into::into)
    }
}
