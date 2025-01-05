use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Visibility};

use crate::model::Model;

use super::index::ByFnContext;

pub struct ObjectStoreContext<'a> {
    pub vis: &'a Visibility,
    pub ident: Ident,
    pub model_ident: &'a Ident,
    pub indexes: Vec<ByFnContext>,
}

impl<'a> TryFrom<(&'a Model, Vec<ByFnContext>)> for ObjectStoreContext<'a> {
    type Error = Error;

    fn try_from((model, indexes): (&'a Model, Vec<ByFnContext>)) -> Result<Self, Self::Error> {
        let ident = match &model.object_store_struct {
            Some(name) => Ident::new(&name.value(), name.span()),
            None => Ident::new(&format!("{}ObjectStore", model.ident), model.ident.span()),
        };

        Ok(Self {
            vis: &model.vis,
            ident,
            model_ident: &model.ident,
            indexes,
        })
    }
}

impl ObjectStoreContext<'_> {
    pub fn expand_object_store_definition(&self) -> TokenStream {
        let vis = self.vis;
        let ident = &self.ident;
        let model_ident = self.model_ident;
        let by_fns = self
            .indexes
            .iter()
            .map(|index| index.expand_by_fn_definition())
            .collect::<Vec<_>>();

        quote! {
            #vis struct #ident<'t> {
                object_store: ::deli::ObjectStore<'t, #model_ident>,
            }

            impl<'t> #ident<'t> {
                #(#by_fns)*
            }

            impl<'t> ::core::ops::Deref for #ident<'t> {
                type Target = ::deli::ObjectStore<'t, #model_ident>;

                fn deref(&self) -> &Self::Target {
                    &self.object_store
                }
            }

            impl<'t> ::core::convert::From<::deli::ObjectStore<'t, #model_ident>> for #ident<'t> {
                fn from(object_store: ::deli::ObjectStore<'t, #model_ident>) -> Self {
                    Self { object_store }
                }
            }
        }
    }
}
