use std::borrow::Cow;

use darling::{ast::Data, Error, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident, LitStr, Visibility};

use crate::model_field::IntoType;
use crate::{field_context::FieldContext, model_field::ModelField};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(deli), supports(struct_named))]
pub struct Model {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub name: Option<LitStr>,
    pub store_name: Option<LitStr>,
    pub cursor_name: Option<LitStr>,
    pub key_cursor_name: Option<LitStr>,
    pub data: Data<(), ModelField>,
}

impl Model {
    /// Returns all the fields of struct
    pub fn fields(&self) -> &[ModelField] {
        match self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(ref fields) => &fields.fields,
        }
    }

    /// Returns the model name
    pub fn model_name(&self) -> Cow<'_, LitStr> {
        match self.name {
            Some(ref name) => Cow::Borrowed(name),
            None => Cow::Owned(LitStr::new(&self.ident.to_string(), self.ident.span())),
        }
    }

    /// Returns the store struct name
    pub fn store_name(&self) -> Ident {
        match self.store_name {
            None => Ident::new(&format!("{}Store", self.ident), self.ident.span()),
            Some(ref store_name) => Ident::new(&store_name.value(), store_name.span()),
        }
    }

    /// Returns the cursor struct name
    pub fn cursor_name(&self) -> Ident {
        match self.cursor_name {
            None => Ident::new(&format!("{}Cursor", self.ident), self.ident.span()),
            Some(ref cursor_name) => Ident::new(&cursor_name.value(), cursor_name.span()),
        }
    }

    /// Returns the key cursor struct name
    pub fn key_cursor_name(&self) -> Ident {
        match self.key_cursor_name {
            None => Ident::new(&format!("{}KeyCursor", self.ident), self.ident.span()),
            Some(ref key_cursor_name) => {
                Ident::new(&key_cursor_name.value(), key_cursor_name.span())
            }
        }
    }

    /// Builds the output of derive macro
    pub fn expand(&self) -> TokenStream {
        let field_context = match FieldContext::new(&self.ident, self.fields()) {
            Ok(field_context) => field_context,
            Err(err) => return err.write_errors(),
        };

        match self.impl_model(&field_context) {
            Ok(impl_model) => impl_model,
            Err(err) => err.write_errors(),
        }
    }

    /// Returns the token stream for implementing `Model` trait
    fn impl_model(&self, field_context: &FieldContext<'_>) -> Result<TokenStream, Error> {
        let ident = &self.ident;
        let model_name = self.model_name();
        let vis = &self.vis;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let key_type = field_context.keys.into_type();
        let object_store = field_context.object_store();
        let indexes = field_context.indexes();

        let store_name = self.store_name();
        let cursor_name = self.cursor_name();
        let key_cursor_name = self.key_cursor_name();

        let add_fn = field_context.add_fn()?;
        let update_fn = field_context.update_fn()?;

        let by_index_fns = field_context.by_index_fns();

        Ok(quote! {
            impl #impl_generics ::deli::Model for #ident #ty_generics #where_clause {
                const NAME: &'static str = #model_name;

                type Key = #key_type;

                type Store<'transaction> = #store_name <'transaction>;

                type Cursor<'transaction> = #cursor_name <'transaction>;

                type KeyCursor<'transaction> = #key_cursor_name <'transaction>;

                fn handle_upgrade(event: ::deli::VersionChangeEvent) {
                    use idb::DatabaseEvent;
                    let database = event.database().unwrap();
                    #object_store
                    #(#indexes)*
                }
            }

            #[derive(Debug)]
            #vis struct #store_name <'transaction> {
                store: ::deli::Store<'transaction, #ident>,
            }

            impl<'transaction> ::core::ops::Deref for #store_name <'transaction> {
                type Target = ::deli::Store<'transaction, #ident>;

                fn deref(&self) -> &Self::Target {
                    &self.store
                }
            }

            impl<'transaction> ::core::convert::From<::deli::Store<'transaction, #ident>> for #store_name <'transaction> {
                fn from(store: ::deli::Store<'transaction, #ident>) -> Self {
                    Self { store }
                }
            }

            impl<'transaction> #store_name <'transaction> {
                #add_fn
                #update_fn

                #(#by_index_fns)*
            }

            #[derive(Debug)]
            #vis struct #cursor_name <'transaction> {
                cursor: ::deli::Cursor<'transaction, #ident>,
            }

            impl<'transaction> ::core::ops::Deref for #cursor_name <'transaction> {
                type Target = ::deli::Cursor<'transaction, #ident>;

                fn deref(&self) -> &Self::Target {
                    &self.cursor
                }
            }

            impl<'transaction> ::core::ops::DerefMut for #cursor_name <'transaction> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.cursor
                }
            }

            impl<'transaction> ::core::convert::From<::deli::Cursor<'transaction, #ident>> for #cursor_name <'transaction> {
                fn from(cursor: ::deli::Cursor<'transaction, #ident>) -> Self {
                    Self { cursor }
                }
            }

            #[derive(Debug)]
            #vis struct #key_cursor_name <'transaction> {
                cursor: ::deli::KeyCursor<'transaction, #ident>,
            }

            impl<'transaction> ::core::ops::Deref for #key_cursor_name <'transaction> {
                type Target = ::deli::KeyCursor<'transaction, #ident>;

                fn deref(&self) -> &Self::Target {
                    &self.cursor
                }
            }

            impl<'transaction> ::core::ops::DerefMut for #key_cursor_name <'transaction> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.cursor
                }
            }

            impl<'transaction> ::core::convert::From<::deli::KeyCursor<'transaction, #ident>> for #key_cursor_name <'transaction> {
                fn from(cursor: ::deli::KeyCursor<'transaction, #ident>) -> Self {
                    Self { cursor }
                }
            }
        })
    }
}
