use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, TypePath};

use crate::model_field::ModelField;

/// Returns the generic parameter of an ident
fn get_generic_param(ident: &Ident) -> Result<TypePath, Error> {
    let s = ident.to_string();

    let mut capitalize = true;

    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    syn::parse_str(&result).map_err(Into::into)
}

/// Returns the token stream for function fields signature
pub fn fn_signature(
    fields: &[&ModelField],
) -> Result<(TokenStream, TokenStream, TokenStream), Error> {
    let mut signatures = Vec::with_capacity(fields.len());
    let mut generics = Vec::with_capacity(fields.len());
    let mut where_clauses = Vec::with_capacity(fields.len());

    for field in fields.iter() {
        let field_name = field.ident();
        let field_type = &field.ty;
        let generic = get_generic_param(field_name)?;

        let field_signature = quote! { #field_name : & #generic };
        let where_clause = quote! { #field_type: ::core::borrow::Borrow<#generic>, #generic: ::deli::reexports::serde::Serialize + ?::core::marker::Sized };

        signatures.push(field_signature);
        where_clauses.push(where_clause);
        generics.push(generic);
    }

    let signature = quote! {
        #(#signatures),*
    };
    let generics = quote! { #(#generics),* };
    let where_clause = quote! { where #(#where_clauses),* };

    Ok((generics, signature, where_clause))
}

/// Returns the token stream for creation of json value from creation fields
pub fn fields_json(fields: &[&ModelField]) -> TokenStream {
    let mut result = Vec::with_capacity(fields.len());

    for field in fields.iter() {
        let field_ident = field.ident();
        let field_name = field.name();

        let field_tokens = quote! { #field_name : ::core::borrow::Borrow::borrow(#field_ident) };

        result.push(field_tokens);
    }

    quote! {
        ::deli::reexports::serde_json::json!({
            #(#result),*
        })
    }
}
