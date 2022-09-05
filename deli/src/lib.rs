//! Deli is a convenience wrapper on [`idb`] create for easily creating and managing object stores in an indexed db
//! using derive macros.
mod database;
mod error;
mod model;
mod store;
mod transaction;

#[doc(hidden)]
pub use idb;
#[doc(hidden)]
pub use serde;
#[doc(hidden)]
pub use serde_json;
#[doc(hidden)]
pub use serde_wasm_bindgen;

pub use idb::VersionChangeEvent;

pub use self::{
    database::{Database, DatabaseBuilder},
    error::Error,
    model::Model,
    store::Store,
    transaction::{Transaction, TransactionBuilder},
};

pub use deli_derive::Model;
