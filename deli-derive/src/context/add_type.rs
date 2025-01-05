use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, Visibility};

use crate::model::Model;

use super::KeyContext;

pub enum AddTypeContext<'a> {
    None,
    Some {
        vis: &'a Visibility,
        ident: Ident,
        attrs: &'a [Attribute],
        fields: Vec<AddTypeFieldContext<'a>>,
    },
}

impl AddTypeContext<'_> {
    pub fn ident(&self) -> TokenStream {
        match self {
            AddTypeContext::None => quote! { Self },
            AddTypeContext::Some { ident, .. } => quote! { #ident },
        }
    }

    pub fn expand_add_type_definition(&self) -> TokenStream {
        match self {
            AddTypeContext::None => quote! {},
            AddTypeContext::Some {
                vis,
                ident,
                attrs,
                fields,
            } => {
                let fields = fields
                    .iter()
                    .map(AddTypeFieldContext::expand_field_definition);

                quote! {
                    #[derive(::deli::reexports::serde::Serialize)]
                    #(#attrs)*
                    #vis struct #ident {
                        #(#fields),*
                    }
                }
            }
        }
    }
}

impl<'a> TryFrom<(&'a Model, &'_ KeyContext<'_>)> for AddTypeContext<'a> {
    type Error = Error;

    fn try_from(
        (model, key_context): (&'a Model, &'_ KeyContext<'_>),
    ) -> Result<Self, Self::Error> {
        if !key_context.is_auto_increment() {
            return Ok(AddTypeContext::None);
        }

        let vis = &model.vis;
        let ident = match &model.add_struct_name {
            Some(name) => Ident::new(&name.value(), name.span()),
            None => Ident::new(&format!("Add{}", model.ident), model.ident.span()),
        };
        let attrs = &model.attrs;

        let fields = model
            .fields()
            .iter()
            .filter(|field| !field.auto_increment.is_present())
            .map(|field| AddTypeFieldContext {
                ident: field.ident.as_ref().unwrap(),
                ty: &field.ty,
                attrs: &field.attrs,
            })
            .collect::<Vec<_>>();

        Ok(Self::Some {
            vis,
            ident,
            attrs,
            fields,
        })
    }
}

pub struct AddTypeFieldContext<'a> {
    pub ident: &'a Ident,
    pub ty: &'a syn::Type,
    pub attrs: &'a [Attribute],
}

impl AddTypeFieldContext<'_> {
    fn expand_field_definition(&self) -> TokenStream {
        let ident = self.ident;
        let ty = self.ty;
        let attrs = self.attrs;

        quote! {
            #(#attrs)*
            pub #ident: #ty
        }
    }
}
