use std::borrow::Cow;

use darling::{util::Flag, FromField};
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
