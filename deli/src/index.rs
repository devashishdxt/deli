use std::borrow::Borrow;

use idb::{CursorDirection, Query};
use serde::Serialize;

use crate::{
    cursor::Cursor,
    error::Error,
    key_cursor::KeyCursor,
    key_range::{BoundedRange, KeyRange, UnboundedRange},
    model::Model,
    model_index::ModelIndex,
    transaction::Transaction,
};

/// Provides access to an index in a database.
#[derive(Debug)]
pub struct Index<'t, I> {
    index: idb::Index,
    transaction: &'t Transaction,
    _model: std::marker::PhantomData<I>,
}

impl<'t, I> Index<'t, I>
where
    I: ModelIndex,
{
    pub(crate) fn new(object_store: idb::Index, transaction: &'t Transaction) -> Self {
        Self {
            index: object_store,
            transaction,
            _model: std::marker::PhantomData,
        }
    }

    /// Retrieves the value of the first record matching the given key range.
    pub async fn get<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, BoundedRange>>,
    ) -> Result<Option<I::Model>, Error>
    where
        I::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.index
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
    ) -> Result<Option<<I::Model as Model>::Key>, Error>
    where
        I::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.index
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
    ) -> Result<Vec<I::Model>, Error>
    where
        I::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.index
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
    ) -> Result<Vec<<I::Model as Model>::Key>, Error>
    where
        I::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.index
            .get_all_keys(<Option<Query>>::try_from(&key_range.into())?, limit)?
            .await?
            .into_iter()
            .map(serde_wasm_bindgen::from_value)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    /// Retrieves the number of records matching the given key range.
    pub async fn count<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, UnboundedRange>>,
    ) -> Result<u32, Error>
    where
        I::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        self.index
            .count(<Option<Query>>::try_from(&key_range.into())?)?
            .await
            .map_err(Into::into)
    }

    /// Opens a [`Cursor`] over the records matching key range, ordered by direction.
    pub async fn cursor<'a, Q>(
        &self,
        key_range: impl Into<KeyRange<'a, Q, UnboundedRange>>,
        cursor_direction: Option<CursorDirection>,
    ) -> Result<Option<Cursor<'t, I::Model, I::Key>>, Error>
    where
        I::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        Ok(self
            .index
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
    ) -> Result<Option<KeyCursor<'t, I::Model, I::Key>>, Error>
    where
        I::Key: Borrow<Q>,
        Q: Serialize + ?Sized + 'a,
    {
        Ok(self
            .index
            .open_key_cursor(
                <Option<Query>>::try_from(&key_range.into())?,
                cursor_direction,
            )?
            .await?
            .map(|cursor| KeyCursor::new(cursor.into_managed(), self.transaction)))
    }
}
