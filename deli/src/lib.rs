#![deny(missing_docs)]
#![forbid(unsafe_code, unstable_features)]
//! Deli is a convenience wrapper on `idb` crate for easily creating and managing object stores in an indexed db on
//! browsers using derive macros.
//!
//! # Usage
//!
//! To use `deli`, you need to add the following in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! deli = "0.1"
//! ```
//!
//! `deli` is intended to be used on browsers using webassembly. So, make sure to compile your project with
//! `--target wasm32-unknown-unknown`. Alternatively, you can add following build configuration in your
//! `.cargo/config.toml`:
//!
//! ```toml
//! [build]
//! target = "wasm32-unknown-unknown"
//! ```
//!
//! ## Example
//!
//! ### Defining a `Model`
//!
//! The first step is to define your data model using `Model` derive macro. You also need to implement
//! `serde::Serialize` and `serde::Deserialize` trait for your model so that the data can be converted to `json` before
//! saving it into the store.
//!
//! ```rust
//! use deli::Model;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Model)]
//! pub struct Employee {
//!     #[deli(auto_increment)]
//!     pub id: u32,
//!     pub name: String,
//!     #[deli(unique)]
//!     pub email: String,
//!     #[deli(index)]
//!     pub age: u8,
//! }
//! ```
//!
//! `Model` derive macro automatically implements `Model` trait for your struct and creates a `Store` for accessing and
//! writing data to the store.
//!
//! #### Container attributes
//!
//! - `#[deli(name)]`: In indexed DB, by default, it creates a new object store with name of the struct (in the above
//!   example, it'll create an object store `Employee` in indexed db) when creating a database. To change the default
//!   object store name, use `#[deli(name = "your_object_store_name")]`.
//! - `#[deli(store_name)]`: By default, the derive macro will create a `<ModelName>Store` struct (in the above example,
//!   it'll create a `EmployeeStore` struct). To change the default name, use `#[deli(store_name = "YourStoreName")]`.
//! - `#[deli(cursor_name)]`: By default, the derive macro will create a `<ModelName>Cursor` struct. To change the
//!   default name, use `#[deli(cursor_name = "YourCursorName")]`
//! - `#[deli(key_cursor_name)]`: By default, the derive macro will create a `<ModelName>KeyCursor` struct. To change
//!   the default name, use `#[deli(key_cursor_name = "YourKeyCursorName")]`
//!
//! #### Field attributes
//!
//! - `#[deli(key)]`: Defines the primary key path for object store.
//! - `#[deli(auto_increment)]`: Defines the primary key path for object store with `auto_increment` values (implies
//!   `#[deli(key)]`).
//! - `#[deli(index)]`: Creates an index for the field.
//! - `#[deli(unique)]`: Creates an unique index for the field (implies `#[deli(index)]`).
//! - `#[deli(multi_entry)]`: Creates a multi entry index for the field (implies `#[deli(index)]`).
//! - `#[deli(rename)]`: Rename a field in object store. Note that this should be consistent with `serde` serialization.
//!   For example, if you use `#[serde(rename_all = "camelCase")]` you need to appropriately rename the fields for
//!   `deli` to be in sync with serde serialization.
//!
//! ### Creating a `Database`
//!
//! Next step is to create a new `Database` and register your models with it.
//!
//! ```rust
//! use deli::{Database, Error};
//!
//! async fn create_database() -> Result<Database, Error> {
//!     let database = Database.builder("test_db", 1).register_model::<Employee>().await?;
//! }
//! ```
//!
//! ### Starting a `Transaction`
//!
//! Once you've created a `Database` instance, you can start reading and writing data to database using transactions.
//!
//! ```rust
//! use deli::{Database, Error, Transaction};
//!
//! fn create_read_transaction(database: &Database) -> Result<Transaction, Error> {
//!     database.transaction().with_model::<Employee>().build()
//! }
//!
//! fn create_write_transaction(database: &Database) -> Result<Transaction, Error> {
//!     database.transaction().writable().with_model::<Employee>().build()
//! }
//! ```
//!
//! You can add multiple `.with_model::<Model>()` calls to add more than one model to the transaction.
//!
//! ### Reading and writing data to a `Model` store
//!
//! Once you have a transaction for a model, you can read or write data to that model. `Model` derive macro generates a
//! static method on the model struct named `with_transaction` which can be used to obtain store for that model.
//!
//! ```rust
//! use deli::{Error, Transaction};
//!
//! async fn add_employee(transaction: &Transaction) -> Result<u32, Error> {
//!     Employee::with_transaction(transaction)?.add("Alice", "alice@example.com", &25).await
//! }
//!
//! async fn get_employee(transaction: &Transaction, id: u32) -> Result<Option<Employee>, Error> {
//!     Employee::with_transaction(transaction)?.get(&id).await
//! }
//!
//! async fn get_all_employees(transaction: &Transaction) -> Result<Vec<Employee>, Error> {
//!     // NOTE: Here `..` (i.e., `RangeFull`) means fetch all values from store
//!     Employee::with_transaction(transaction)?.get_all(.., None).await
//! }
//!
//! async fn get_employees_with_bounds(
//!     transaction: &Transaction,
//!     from_id: u32,
//!     to_id: u32,
//! ) -> Result<Vec<Employee>, Error> {
//!     Employee::with_transaction(transaction)?.get_all(&from_id..=&to_id, None).await
//! }
//! ```
//!
//! ### Commiting a `Transaction`
//!
//! After all your writes are done, you can commit the transaction:
//!
//! ```rust
//! async fn commit_transaction(transaction: Transaction) -> Result<(), Error> {
//!     transaction.commit().await
//! }
//! ```
//!
//! Note that `commit()` doesn’t normally have to be called — a transaction will automatically commit when all
//! outstanding requests have been satisfied and no new requests have been made.
//!
//! Also, be careful when using long-lived indexed db transactions as the behavior may change depending on the browser.
//! For example, the transaction may get auto-committed when doing IO (network request) in the event loop.
mod cursor;
mod database;
mod error;
mod index;
mod key_cursor;
mod key_range;
mod model;
#[doc(hidden)]
pub mod reexports;
mod specific_key_range;
mod store;
mod transaction;

pub use idb::{CursorDirection as Direction, VersionChangeEvent};

pub use self::{
    cursor::Cursor,
    database::{Database, DatabaseBuilder},
    error::Error,
    index::Index,
    key_cursor::KeyCursor,
    key_range::KeyRange,
    model::Model,
    specific_key_range::SpecificKeyRange,
    store::Store,
    transaction::{Transaction, TransactionBuilder},
};

pub use deli_derive::Model;
