use darling::{error::Accumulator, Error};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    model_field::ModelField,
    utils::{fields_json, fn_signature},
};

/// Context for storing keys and indexes of object store
#[derive(Debug)]
pub struct FieldContext<'a> {
    pub ident: &'a Ident,
    pub key: &'a ModelField,
    pub indexes: Vec<&'a ModelField>,
    pub creation_fields: Vec<&'a ModelField>,
    pub updation_fields: Vec<&'a ModelField>,
}

impl<'a> FieldContext<'a> {
    /// Creates a new field context
    pub fn new(ident: &'a Ident, fields: &'a [ModelField]) -> Result<Self, Error> {
        let mut builder = FieldContextBuilder::new(ident);

        for field in fields {
            builder.with_field(field);
        }

        builder.build()
    }

    /// Returns the token stream for creating indexes
    pub fn indexes(&self) -> Vec<TokenStream> {
        let mut indexes = Vec::with_capacity(self.indexes.len());

        for field in self.indexes.iter() {
            let name = field.name();
            let key_path = quote! { ::deli::idb::KeyPath::new_single(#name) };

            let index_params = if field.unique.is_present() || field.multi_entry.is_present() {
                let unique = if field.unique.is_present() {
                    quote! { params.unique(true); }
                } else {
                    quote! {}
                };

                let multi_entry = if field.multi_entry.is_present() {
                    quote! { params.multi_entry(true); }
                } else {
                    quote! {}
                };

                quote! {
                    Some({
                        let mut params = ::deli::idb::IndexParams::new();
                        #unique
                        #multi_entry
                        params
                    })
                }
            } else {
                quote! { None }
            };

            let index = quote! {
                object_store
                    .create_index(#name, #key_path, #index_params)
                    .unwrap();
            };

            indexes.push(index);
        }

        indexes
    }

    /// Returns the token stream for creating object store
    pub fn object_store(&self) -> TokenStream {
        let store_params = self.store_params();

        if self.indexes.is_empty() {
            quote! {
                database
                    .create_object_store(<Self as ::deli::Model>::NAME, #store_params)
                    .unwrap();
            }
        } else {
            quote! {
                let object_store = database
                    .create_object_store(<Self as ::deli::Model>::NAME, #store_params)
                    .unwrap();
            }
        }
    }

    /// Returns token stream for add function
    pub fn add_fn(&self, ident: &Ident) -> Result<TokenStream, Error> {
        let (generics, signature, where_clause) = fn_signature(&self.creation_fields)?;
        let key_type = &self.key.ty;
        let fields_json = fields_json(&self.creation_fields);

        Ok(quote! {
            pub async fn add<#generics>(&self, #signature) -> Result<#key_type, ::deli::Error> #where_clause {
                let value = #fields_json;
                let store = self.transaction.store::<#ident>()?;

                store.add(&value).await
            }
        })
    }

    /// Returns token stream for update function
    pub fn update_fn(&self, ident: &Ident) -> Result<TokenStream, Error> {
        let (generics, signature, where_clause) = fn_signature(&self.updation_fields)?;
        let key_type = &self.key.ty;
        let fields_json = fields_json(&self.updation_fields);

        Ok(quote! {
            pub async fn update<#generics>(&self, #signature) -> Result<#key_type, ::deli::Error> #where_clause {
                let value = #fields_json;
                let store = self.transaction.store::<#ident>()?;

                store.update(&value).await
            }
        })
    }

    /// Returns the token stream for object store params
    fn store_params(&self) -> TokenStream {
        let name = self.key.name();

        let key_path = quote! { params.key_path(Some(::deli::idb::KeyPath::new_single(#name))); };

        let auto_increment = if self.key.auto_increment.is_present() {
            quote! { params.auto_increment(true); }
        } else {
            quote! {}
        };

        quote! {{
            let mut params = ::deli::idb::ObjectStoreParams::new();
            #key_path
            #auto_increment
            params
        }}
    }
}

/// Builder for field context
#[derive(Debug)]
struct FieldContextBuilder<'a> {
    ident: &'a Ident,
    key: Option<&'a ModelField>,
    indexes: Vec<&'a ModelField>,
    creation_fields: Vec<&'a ModelField>,
    updation_fields: Vec<&'a ModelField>,
    accumulator: Accumulator,
}

impl<'a> FieldContextBuilder<'a> {
    fn new(ident: &'a Ident) -> Self {
        Self {
            ident,
            key: Default::default(),
            indexes: Default::default(),
            creation_fields: Default::default(),
            updation_fields: Default::default(),
            accumulator: Default::default(),
        }
    }

    /// Adds a field to builder
    fn with_field(&mut self, field: &'a ModelField) {
        if field.key.is_present() || field.auto_increment.is_present() {
            match self.key {
                Some(_) => self.accumulator.push(
                    Error::custom(format!("multiple keys defined for model {}", self.ident))
                        .with_span(&self.ident.span()),
                ),
                None => self.key = Some(field),
            }
        } else if field.index.is_present()
            || field.unique.is_present()
            || field.multi_entry.is_present()
        {
            self.indexes.push(field);
        }

        if !field.auto_increment.is_present() {
            self.creation_fields.push(field);
        }

        self.updation_fields.push(field);
    }

    /// Builds field context
    fn build(mut self) -> Result<FieldContext<'a>, Error> {
        if self.key.is_none() {
            self.accumulator.push(
                Error::custom(format!("no key defined for model {}", self.ident))
                    .with_span(&self.ident.span()),
            );
        }

        self.accumulator.finish()?;

        let key = self.key.unwrap(); // We just checked this value above and pushed the error in accumulator

        Ok(FieldContext {
            ident: self.ident,
            key,
            indexes: self.indexes,
            creation_fields: self.creation_fields,
            updation_fields: self.updation_fields,
        })
    }
}
