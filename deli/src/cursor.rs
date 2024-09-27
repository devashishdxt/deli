use std::marker::PhantomData;

use crate::{Error, Model, Transaction};

/// Cursor on an object store or index
#[derive(Debug)]
pub struct Cursor<'t, M>
where
    M: Model,
{
    cursor: idb::ManagedCursor,
    _transaction: &'t Transaction,
    _generics_model: PhantomData<M>,
}

impl<'t, M> Cursor<'t, M>
where
    M: Model,
{
    /// Creates a new instance of cursor
    pub(crate) fn new(transaction: &'t Transaction, cursor: idb::ManagedCursor) -> Self {
        Self {
            cursor,
            _transaction: transaction,
            _generics_model: Default::default(),
        }
    }

    /// Returns the key at current cursor position
    pub fn get_key(&self) -> Result<Option<M::Key>, Error> {
        let js_value = self.cursor.primary_key()?;
        js_value
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .transpose()
    }

    /// Returns the value at current cursor position
    pub fn get_value(&self) -> Result<Option<M>, Error> {
        let js_value = self.cursor.value()?;
        js_value
            .map(|js_value| serde_wasm_bindgen::from_value(js_value).map_err(Into::into))
            .transpose()
    }

    /// Advances the cursor
    pub async fn advance(&mut self, count: u32) -> Result<(), Error> {
        Ok(self.cursor.advance(count).await?)
    }

    /// Deletes the entry at current cursor position
    pub async fn delete(&self) -> Result<(), Error> {
        Ok(self.cursor.delete().await?)
    }
}
