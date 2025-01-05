use std::borrow::Cow;

use darling::{error::Accumulator, Error};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{LitStr, Type};

use crate::model::Model;

pub enum KeyContext<'a> {
    Single {
        key: Cow<'a, LitStr>,
        ty: &'a Type,
        auto_increment: bool,
    },
    Composite {
        keys: Vec<Cow<'a, LitStr>>,
        tys: Vec<&'a Type>,
    },
}

impl KeyContext<'_> {
    pub fn is_auto_increment(&self) -> bool {
        match self {
            KeyContext::Single { auto_increment, .. } => *auto_increment,
            KeyContext::Composite { .. } => false,
        }
    }

    pub fn expand_object_store_builder(&self) -> TokenStream {
        match self {
            KeyContext::Single {
                key,
                auto_increment,
                ..
            } => {
                let auto_increment = if *auto_increment {
                    quote! { .auto_increment(true) }
                } else {
                    quote! {}
                };

                quote! {
                    .key_path(::core::option::Option::Some(::deli::reexports::idb::KeyPath::new_single( #key )))#auto_increment
                }
            }
            KeyContext::Composite { keys, .. } => {
                quote! {
                    .key_path(::core::option::Option::Some(::deli::reexports::idb::KeyPath::new_array([ #(#keys),* ])))
                }
            }
        }
    }

    pub fn expand_key_type(&self) -> TokenStream {
        match self {
            KeyContext::Single { ty, .. } => {
                quote! { #ty }
            }
            KeyContext::Composite { tys, .. } => {
                quote! { ( #(#tys),* ) }
            }
        }
    }
}

impl<'a> TryFrom<&'a Model> for KeyContext<'a> {
    type Error = Error;

    fn try_from(value: &'a Model) -> Result<Self, Self::Error> {
        get_key(value)
    }
}

fn get_key(model: &Model) -> Result<KeyContext<'_>, Error> {
    let mut accumulator = Accumulator::default();

    let composite_key = get_composite_key(model);
    let single_key = get_single_key(model);

    match (composite_key, single_key) {
        (Ok(Some(_)), Ok(Some(_))) => {
            accumulator.push(
                Error::custom("Model cannot have both composite key and field key")
                    .with_span(&model.ident),
            );
        }
        (Ok(None), Ok(None)) => {
            accumulator.push(
                Error::custom("Model must have either composite key or field key")
                    .with_span(&model.ident),
            );
        }
        (Ok(Some(composite_key)), Ok(None)) => {
            accumulator.finish()?;
            return Ok(composite_key);
        }
        (Ok(None), Ok(Some(single_key))) => {
            accumulator.finish()?;
            return Ok(single_key);
        }
        (err1, err2) => {
            if let Err(err) = err1 {
                accumulator.push(err);
            }

            if let Err(err) = err2 {
                accumulator.push(err);
            }
        }
    }

    Err(accumulator.finish().unwrap_err())
}

fn get_composite_key(model: &Model) -> Result<Option<KeyContext<'_>>, Error> {
    match model.key.as_ref() {
        None => Ok(None),
        Some(path_list) => {
            let (keys, tys) = model
                .get_fields_from_path_list(path_list)?
                .into_iter()
                .map(|field| (field.get_name_str(), &field.ty))
                .unzip();

            Ok(Some(KeyContext::Composite { keys, tys }))
        }
    }
}

fn get_single_key(model: &Model) -> Result<Option<KeyContext<'_>>, Error> {
    let field = model
        .fields()
        .iter()
        .filter(|field| field.is_key())
        .collect::<Vec<_>>();

    if field.len() > 1 {
        return Err(
            Error::custom("Model cannot have more than one field key").with_span(&model.ident)
        );
    }

    if field.is_empty() {
        return Ok(None);
    }

    let field = field.first().unwrap();

    Ok(Some(KeyContext::Single {
        key: field.get_name_str(),
        auto_increment: field.auto_increment.is_present(),
        ty: &field.ty,
    }))
}
