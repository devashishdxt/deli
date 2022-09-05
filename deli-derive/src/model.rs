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

    /// Builds the output of derive macro
    pub fn expand(&self) -> TokenStream {
        let field_context = match FieldContext::new(self.fields()) {
            Ok(field_context) => field_context,
            Err(err) => return err.write_errors(),
        };

        let impl_model = self.impl_model(&field_context);
        let impl_db = match self.impl_db(&field_context) {
            Ok(impl_db) => impl_db,
            Err(err) => return err.write_errors(),
        };

        quote! {
            #impl_model
            #impl_db
        }
    }

    /// Returns the token stream for implementing `Model` trait
    fn impl_model(&self, field_context: &FieldContext<'_>) -> TokenStream {
        let ident = &self.ident;
        let model_name = self.model_name();

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let key_type = &field_context.key.ty;
        let object_store = field_context.object_store();
        let indexes = field_context.indexes();

        quote! {
            impl #impl_generics ::deli::Model for #ident #ty_generics #where_clause {
                const NAME: &'static str = #model_name;

                type Key = #key_type;

                fn handle_upgrade(event: ::deli::VersionChangeEvent) {
                    let database = event.database().unwrap();
                    #object_store
                    #(#indexes)*
                }
            }
        }
    }

    /// Returns the token stream for implementing `ModelDb` struct
    fn impl_db(&self, field_context: &FieldContext<'_>) -> Result<TokenStream, Error> {
        let ident = &self.ident;
        let transaction_name = Ident::new(&format!("{}Transaction", self.ident), self.ident.span());
        let vis = &self.vis;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let add_fn = field_context.add_fn(ident)?;
        let update_fn = field_context.update_fn(ident)?;

        Ok(quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                pub fn with_transaction(transaction: &::deli::Transaction) -> #transaction_name<'_> {
                    #transaction_name ::new(transaction)
                }
            }

            #vis struct #transaction_name <'deli> {
                transaction: &'deli ::deli::Transaction,
            }

            impl<'deli> #transaction_name <'deli> {
                pub fn new(transaction: &'deli ::deli::Transaction) -> Self {
                    Self { transaction: transaction }
                }

                #add_fn
                #update_fn
            }
        })
    }
}
