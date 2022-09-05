#![deny(missing_docs)]
#![forbid(unsafe_code, unstable_features)]
//! This crate implements derive macro for implementing `deli::Model` trait on named structs.
mod field_context;
mod model;
mod model_field;
mod utils;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use self::model::Model;

/// Derive macro for implementing `Model` trait on structs
#[proc_macro_derive(Model, attributes(deli))]
pub fn model(item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let derive_input = parse_macro_input!(item as DeriveInput);

    // Create a model from derive input
    let model = match Model::from_derive_input(&derive_input) {
        Ok(model) => model,
        Err(err) => return err.write_errors().into(),
    };

    // Return the output of derive macro
    model.expand().into()
}
