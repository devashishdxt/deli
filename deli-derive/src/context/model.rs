use std::borrow::Cow;

use darling::{error::Accumulator, Error};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::model::Model;

use super::{add_type::AddTypeContext, object_store::ObjectStoreContext, IndexContext, KeyContext};

pub struct ModelContext<'a> {
    pub ident: &'a Ident,
    pub name: Cow<'a, LitStr>,
    pub key: KeyContext<'a>,
    pub indexes: Vec<IndexContext<'a>>,
    pub add_type: AddTypeContext<'a>,
    pub object_store: ObjectStoreContext<'a>,
}

impl ModelContext<'_> {
    pub fn expand(&self) -> TokenStream {
        let model_definition = self.expand_model_definition();
        let add_type_definition = self.add_type.expand_add_type_definition();
        let index_definitions = self
            .indexes
            .iter()
            .map(|index| index.expand_model_index_definition());
        let object_store_definition = self.object_store.expand_object_store_definition();

        quote! {
            #model_definition

            #add_type_definition

            #(#index_definitions)*

            #object_store_definition
        }
    }

    fn expand_model_definition(&self) -> TokenStream {
        let ident = self.ident;
        let name = &self.name;
        let key = self.key.expand_key_type();
        let add = &self.add_type.ident();
        let object_store = &self.object_store.ident;

        let key_object_store_builder = self.key.expand_object_store_builder();
        let indexes_object_store_builder = self
            .indexes
            .iter()
            .map(|index| index.expand_object_store_builder());

        quote! {
            impl ::deli::Model for #ident {
                const NAME: &str = #name;

                type Key = #key;

                type Add = #add;

                type ObjectStore<'t> = #object_store<'t>;

                fn object_store_builder() -> ::deli::reexports::idb::builder::ObjectStoreBuilder {
                    ::deli::reexports::idb::builder::ObjectStoreBuilder::new(Self::NAME)
                        #key_object_store_builder
                        #(#indexes_object_store_builder)*
                }
            }
        }
    }
}

impl<'a> TryFrom<&'a Model> for ModelContext<'a> {
    type Error = Error;

    fn try_from(model: &'a Model) -> Result<Self, Self::Error> {
        let mut accumulator = Accumulator::default();

        let ident = &model.ident;
        let name = model.get_name_str();
        let key = KeyContext::try_from(model);
        let indexes = <Vec<IndexContext<'_>>>::try_from(model);

        let key = match key {
            Ok(key) => Some(key),
            Err(err) => {
                accumulator.push(err);
                None
            }
        };

        let indexes = match indexes {
            Ok(indexes) => Some(indexes),
            Err(err) => {
                accumulator.push(err);
                None
            }
        };

        accumulator.finish()?;

        let key = key.unwrap();
        let indexes = indexes.unwrap();

        let by_fns = indexes
            .iter()
            .map(|index| index.by_fn_context())
            .collect::<Vec<_>>();

        let mut accumulator = Accumulator::default();

        let add_type = AddTypeContext::try_from((model, &key));
        let object_store = ObjectStoreContext::try_from((model, by_fns));

        let add_type = match add_type {
            Ok(add_type) => Some(add_type),
            Err(err) => {
                accumulator.push(err);
                None
            }
        };

        let object_store = match object_store {
            Ok(object_store) => Some(object_store),
            Err(err) => {
                accumulator.push(err);
                None
            }
        };

        accumulator.finish()?;

        let add_type = add_type.unwrap();
        let object_store = object_store.unwrap();

        Ok(Self {
            ident,
            name,
            key,
            indexes,
            add_type,
            object_store,
        })
    }
}
