use std::borrow::Cow;

use darling::{ast::Data, Error, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident, LitStr, Visibility};

use crate::{field_context::FieldContext, model_field::ModelField};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(deli), supports(struct_named))]
pub struct Model {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub name: Option<LitStr>,
    pub store_name: Option<LitStr>,
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

        let key_type = &field_context.key.ty;
        let object_store = field_context.object_store();
        let indexes = field_context.indexes();

        let store_name = self.store_name();

        let add_fn = field_context.add_fn()?;
        let update_fn = field_context.update_fn()?;

        let by_index_fns = field_context.by_index_fns();

        Ok(quote! {
            impl #impl_generics ::deli::Model for #ident #ty_generics #where_clause {
                const NAME: &'static str = #model_name;

                type Key = #key_type;

                type Store = #store_name;

                fn handle_upgrade(event: ::deli::VersionChangeEvent) {
                    let database = event.database().unwrap();
                    #object_store
                    #(#indexes)*
                }
            }

            #vis struct #store_name {
                store: ::deli::Store<#ident>,
            }

            impl ::core::ops::Deref for #store_name {
                type Target = ::deli::Store<#ident>;

                fn deref(&self) -> &Self::Target {
                    &self.store
                }
            }

            impl ::core::convert::From<::deli::Store<#ident>> for #store_name {
                fn from(store: ::deli::Store<#ident>) -> Self {
                    Self { store }
                }
            }

            impl #store_name {
                #add_fn
                #update_fn

                #(#by_index_fns)*
            }
        })
    }
}
