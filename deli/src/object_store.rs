use std::borrow::Borrow;

use idb::{CursorDirection, Query};
use serde::Serialize;

use crate::{
    cursor::Cursor,
    error::Error,
    index::Index,
    key_cursor::KeyCursor,
    key_range::{BoundedRange, KeyRange, UnboundedRange},
    model::Model,
    model_index::ModelIndex,
    transaction::Transaction,
    JSON_SERIALIZER,
};

/// Represents an object store in a database.
#[derive(Debug)]
pub struct ObjectStore<'t, M> {
    object_store: idb::ObjectStore,
    transaction: &'t Transaction,
    _model: std::marker::PhantomData<M>,
}

impl<'t, M> ObjectStore<'t, M>
where
    M: Model,
{
    pub(crate) fn new(object_store: idb::ObjectStore, transaction: &'t Transaction) -> Self {
        Self {
            object_store,
            transaction,
            _model: std::marker::PhantomData,
        }
    }

    /// Retrieves the value of the first record matching the given key range.
    pub async fn get<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, BoundedRange>>,
    ) -> Result<Option<M>, Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.object_store
            .get(Query::try_from(&key_range.into())?)?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()
            .map_err(Into::into)
    }

    /// Retrieves the key of the first record matching the given key range.
    pub async fn get_key<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, BoundedRange>>,
    ) -> Result<Option<M::Key>, Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.object_store
            .get_key(Query::try_from(&key_range.into())?)?
            .await?
            .map(serde_wasm_bindgen::from_value)
            .transpose()
            .map_err(Into::into)
    }

    /// Retrieves all the values of the records matching the given key range (up to limit if given).
    pub async fn get_all<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, UnboundedRange>>,
        limit: Option<u32>,
    ) -> Result<Vec<M>, Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.object_store
            .get_all(<Option<Query>>::try_from(&key_range.into())?, limit)?
            .await?
            .into_iter()
            .map(serde_wasm_bindgen::from_value)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    /// Retrieves all the keys of the records matching the given key range (up to limit if given).
    pub async fn get_all_keys<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, UnboundedRange>>,
        limit: Option<u32>,
    ) -> Result<Vec<M::Key>, Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.object_store
            .get_all_keys(<Option<Query>>::try_from(&key_range.into())?, limit)?
            .await?
            .into_iter()
            .map(serde_wasm_bindgen::from_value)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    /// Adds a record to the store returning its key
    pub async fn add(&self, value: &M::Add) -> Result<M::Key, Error> {
        let value = value.serialize(&JSON_SERIALIZER)?;
        let js_key = self.object_store.add(&value, None)?.await?;
        serde_wasm_bindgen::from_value(js_key).map_err(Into::into)
    }

    /// Updates a record in the store returning its key
    pub async fn update<V>(&self, value: &V) -> Result<M::Key, Error>
    where
        M: Borrow<V>,
        V: Serialize,
    {
        let value = value.serialize(&JSON_SERIALIZER)?;
        let js_key = self.object_store.put(&value, None)?.await?;
        serde_wasm_bindgen::from_value(js_key).map_err(Into::into)
    }

    /// Deletes records in store with the given key range.
    pub async fn delete<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, BoundedRange>>,
    ) -> Result<(), Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.object_store
            .delete(Query::try_from(&key_range.into())?)?
            .await
            .map_err(Into::into)
    }

    /// Clears all records in the store.
    pub async fn delete_all(&self) -> Result<(), Error> {
        self.object_store.clear()?.await.map_err(Into::into)
    }

    /// Retrieves the number of records matching the given key range.
    pub async fn count<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, UnboundedRange>>,
    ) -> Result<u32, Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.object_store
            .count(<Option<Query>>::try_from(&key_range.into())?)?
            .await
            .map_err(Into::into)
    }

    /// Opens a [`Cursor`] over the records matching key range, ordered by direction.
    pub async fn cursor<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, UnboundedRange>>,
        cursor_direction: Option<CursorDirection>,
    ) -> Result<Option<Cursor<'t, M, M::Key>>, Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        Ok(self
            .object_store
            .open_cursor(
                <Option<Query>>::try_from(&key_range.into())?,
                cursor_direction,
            )?
            .await?
            .map(|cursor| Cursor::new(cursor.into_managed(), self.transaction)))
    }

    /// Opens a [`KeyCursor`] over the records matching key range, ordered by direction.
    pub async fn key_cursor<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, UnboundedRange>>,
        cursor_direction: Option<CursorDirection>,
    ) -> Result<Option<KeyCursor<'t, M, M::Key>>, Error>
    where
        M::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        Ok(self
            .object_store
            .open_key_cursor(
                <Option<Query>>::try_from(&key_range.into())?,
                cursor_direction,
            )?
            .await?
            .map(|cursor| KeyCursor::new(cursor.into_managed(), self.transaction)))
    }

    /// Returns an [`Index`] for the given model index.
    #[doc(hidden)]
    pub fn index<I>(&self) -> Result<Index<'t, I>, Error>
    where
        I: ModelIndex<Model = M>,
    {
        Ok(Index::new(
            self.object_store.index(I::NAME)?,
            self.transaction,
        ))
    }
}
