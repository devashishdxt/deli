use darling::{util::PathList, FromMeta};
use syn::LitStr;

#[derive(Debug, FromMeta)]
pub struct ModelIndexMeta {
    #[darling(default)]
    pub name: Option<LitStr>,
    pub fields: PathList,
    #[darling(default)]
    pub struct_name: Option<LitStr>,
}

#[derive(Debug, Default, FromMeta)]
pub struct FieldIndexMeta {
    #[darling(default)]
    pub name: Option<LitStr>,
    #[darling(default)]
    pub struct_name: Option<LitStr>,
}
