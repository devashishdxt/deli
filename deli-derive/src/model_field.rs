use std::borrow::Cow;

use darling::{util::Flag, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Type};

#[derive(Debug, FromField)]
#[darling(attributes(deli))]
pub struct ModelField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default)]
    pub rename: Option<LitStr>,
    #[darling(default)]
    pub key: Flag,
    #[darling(default)]
    pub auto_increment: Flag,
    #[darling(default)]
    pub index: Flag,
    #[darling(default)]
    pub unique: Flag,
    #[darling(default)]
    pub multi_entry: Flag,
}

impl ModelField {
    /// Returns the identifier of the field
    pub fn ident(&self) -> &Ident {
        self.ident.as_ref().unwrap()
    }

    /// Returns the name of the field
    pub fn name(&self) -> Cow<'_, LitStr> {
        match self.rename {
            None => {
                let ident = self.ident();
                Cow::Owned(LitStr::new(&ident.to_string(), ident.span()))
            }
            Some(ref rename) => Cow::Borrowed(rename),
        }
    }
}

pub trait IntoType {
    fn into_type(&self) -> TokenStream;
}

impl IntoType for ModelField {
    fn into_type(&self) -> TokenStream {
        let key_type = self.ty.clone();
        quote! { #key_type }
    }
}

impl IntoType for &ModelField {
    fn into_type(&self) -> TokenStream {
        let key_type = self.ty.clone();
        quote! { #key_type }
    }
}

impl<T: IntoType> IntoType for Vec<T> {
    fn into_type(&self) -> TokenStream {
        if self.is_empty() {
            panic!("Cannot convert empty vector to type");
        }
        if self.len() == 1 {
            self[0].into_type()
        } else {
            let mut types = self
                .iter()
                .map(|field| field.into_type())
                .collect::<Vec<_>>();
            let first = types.remove(0);
            let rest = types.iter().map(|ty| {
                quote! { #ty }
            });
            quote! { (#first, #(#rest),*) }
        }
    }
}
