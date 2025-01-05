mod context;
mod index_meta;
mod model;
mod model_field;

use context::ModelContext;
use darling::FromDeriveInput;
use model::Model;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

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

    // Validate that the model does not contain any generics
    match model.validate_no_generic() {
        Ok(_) => {}
        Err(err) => return err.write_errors().into(),
    }

    // Create a model context from model
    let model_context = match ModelContext::try_from(&model) {
        Ok(model_context) => model_context,
        Err(err) => return err.write_errors().into(),
    };

    // Return the output of derive macro
    model_context.expand().into()
}
