use std::borrow::Cow;

use darling::{
    util::{Flag, Override},
    FromField,
};
use syn::{Attribute, Ident, LitStr, Type, Visibility};

use crate::index_meta::FieldIndexMeta;

#[derive(Debug, FromField)]
#[darling(attributes(deli), forward_attrs(allow, doc, serde))]
pub struct ModelField {
    pub ident: Option<Ident>,
    pub vis: Visibility,
    pub ty: Type,
    #[darling(default)]
    pub rename: Option<LitStr>,
    #[darling(default)]
    pub key: Flag,
    #[darling(default)]
    pub auto_increment: Flag,
    #[darling(default)]
    pub index: Option<Override<FieldIndexMeta>>,
    #[darling(default)]
    pub unique: Option<Override<FieldIndexMeta>>,
    #[darling(default)]
    pub multi_entry: Option<Override<FieldIndexMeta>>,
    pub attrs: Vec<Attribute>,
}

impl ModelField {
    pub fn ident(&self) -> &Ident {
        self.ident.as_ref().unwrap()
    }

    pub fn is_key(&self) -> bool {
        self.key.is_present() || self.auto_increment.is_present()
    }

    pub fn is_index(&self) -> bool {
        self.index.is_some() || self.unique.is_some() || self.multi_entry.is_some()
    }

    pub fn get_name_str(&self) -> Cow<'_, LitStr> {
        match &self.rename {
            Some(rename) => Cow::Borrowed(rename),
            None => {
                let ident = self.ident.as_ref().unwrap();
                Cow::Owned(LitStr::new(&ident.to_string(), ident.span()))
            }
        }
    }
}
