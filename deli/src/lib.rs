#![deny(missing_docs)]
#![forbid(unsafe_code, unstable_features)]
//! Deli is a convenience wrapper on [`idb`] create for easily creating and managing object stores in an indexed db
//! using derive macros.
mod database;
mod error;
mod model;
pub mod reexports;
mod store;
mod transaction;

pub use idb::VersionChangeEvent;

pub use self::{
    database::{Database, DatabaseBuilder},
    error::Error,
    model::Model,
    store::Store,
    transaction::{Transaction, TransactionBuilder},
};

pub use deli_derive::Model;
