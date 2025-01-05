use std::borrow::Cow;

use darling::{error::Accumulator, util::Override, Error};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Type, Visibility};

use crate::{index_meta::ModelIndexMeta, model::Model, model_field::ModelField};

pub struct ByFnContext {
    pub index_ident: Ident,
    pub by_fn_ident: Ident,
}

impl ByFnContext {
    pub fn expand_by_fn_definition(&self) -> TokenStream {
        let index_ident = &self.index_ident;
        let by_fn_ident = &self.by_fn_ident;

        quote! {
            pub fn #by_fn_ident(&self) -> ::core::result::Result<::deli::Index<'t, #index_ident>, ::deli::Error> {
                self.index::<#index_ident>()
            }
        }
    }
}

pub enum IndexContext<'a> {
    Single {
        vis: &'a Visibility,
        key: Cow<'a, LitStr>,
        index_ident: Ident,
        index_name: Cow<'a, LitStr>,
        index_model: &'a Ident,
        index_ty: &'a Type,
        by_fn_ident: Ident,
    },
    SingleUnique {
        vis: &'a Visibility,
        key: Cow<'a, LitStr>,
        index_ident: Ident,
        index_name: Cow<'a, LitStr>,
        index_model: &'a Ident,
        index_ty: &'a Type,
        by_fn_ident: Ident,
    },
    SingleMultiEntry {
        vis: &'a Visibility,
        key: Cow<'a, LitStr>,
        index_ident: Ident,
        index_name: Cow<'a, LitStr>,
        index_model: &'a Ident,
        index_ty: &'a Type,
        by_fn_ident: Ident,
    },
    Composite {
        vis: &'a Visibility,
        keys: Vec<Cow<'a, LitStr>>,
        index_ident: Ident,
        index_name: Cow<'a, LitStr>,
        index_model: &'a Ident,
        index_tys: Vec<&'a Type>,
        by_fn_ident: Ident,
    },
    CompositeUnique {
        vis: &'a Visibility,
        keys: Vec<Cow<'a, LitStr>>,
        index_ident: Ident,
        index_name: Cow<'a, LitStr>,
        index_model: &'a Ident,
        index_tys: Vec<&'a Type>,
        by_fn_ident: Ident,
    },
    CompositeMultiEntry {
        vis: &'a Visibility,
        keys: Vec<Cow<'a, LitStr>>,
        index_ident: Ident,
        index_name: Cow<'a, LitStr>,
        index_model: &'a Ident,
        index_tys: Vec<&'a Type>,
        by_fn_ident: Ident,
    },
}

impl<'a> TryFrom<&'a Model> for Vec<IndexContext<'a>> {
    type Error = Error;

    fn try_from(value: &'a Model) -> Result<Self, Self::Error> {
        get_indexes(value)
    }
}

impl IndexContext<'_> {
    fn ident(&self) -> &Ident {
        match self {
            IndexContext::Single { index_ident, .. }
            | IndexContext::SingleUnique { index_ident, .. }
            | IndexContext::SingleMultiEntry { index_ident, .. }
            | IndexContext::Composite { index_ident, .. }
            | IndexContext::CompositeUnique { index_ident, .. }
            | IndexContext::CompositeMultiEntry { index_ident, .. } => index_ident,
        }
    }

    pub fn expand_object_store_builder(&self) -> TokenStream {
        let ident = self.ident();
        quote! { .add_index( <#ident as ::deli::ModelIndex> ::index_builder()) }
    }

    pub fn by_fn_context(&self) -> ByFnContext {
        let (index_ident, by_fn_ident) = match self {
            IndexContext::Single {
                index_ident,
                by_fn_ident,
                ..
            }
            | IndexContext::SingleUnique {
                index_ident,
                by_fn_ident,
                ..
            }
            | IndexContext::SingleMultiEntry {
                index_ident,
                by_fn_ident,
                ..
            }
            | IndexContext::Composite {
                index_ident,
                by_fn_ident,
                ..
            }
            | IndexContext::CompositeUnique {
                index_ident,
                by_fn_ident,
                ..
            }
            | IndexContext::CompositeMultiEntry {
                index_ident,
                by_fn_ident,
                ..
            } => (index_ident, by_fn_ident),
        };

        ByFnContext {
            index_ident: index_ident.clone(),
            by_fn_ident: by_fn_ident.clone(),
        }
    }

    pub fn expand_model_index_definition(&self) -> TokenStream {
        match self {
            IndexContext::Single {
                vis,
                key,
                index_ident,
                index_name,
                index_model,
                index_ty,
                ..
            } => {
                quote! {
                    #vis struct #index_ident;

                    impl ::deli::ModelIndex for #index_ident {
                        const NAME: &'static str = #index_name;

                        type Model = #index_model;

                        type Key = #index_ty;

                        fn index_builder() -> ::deli::reexports::idb::builder::IndexBuilder {
                            ::deli::reexports::idb::builder::IndexBuilder::new(
                                ::std::string::ToString::to_string(<Self as ::deli::ModelIndex>::NAME),
                                ::deli::reexports::idb::KeyPath::new_single( #key ),
                            )
                        }
                    }
                }
            }
            IndexContext::SingleUnique {
                vis,
                key,
                index_ident,
                index_name,
                index_model,
                index_ty,
                ..
            } => {
                quote! {
                    #vis struct #index_ident;

                    impl ::deli::ModelIndex for #index_ident {
                        const NAME: &'static str = #index_name;

                        type Model = #index_model;

                        type Key = #index_ty;

                        fn index_builder() -> ::deli::reexports::idb::builder::IndexBuilder {
                            ::deli::reexports::idb::builder::IndexBuilder::new(
                                ::std::string::ToString::to_string(<Self as ::deli::ModelIndex>::NAME),
                                ::deli::reexports::idb::KeyPath::new_single( #key ),
                            )
                            .unique(true)
                        }
                    }
                }
            }
            IndexContext::SingleMultiEntry {
                vis,
                key,
                index_ident,
                index_name,
                index_model,
                index_ty,
                ..
            } => {
                quote! {
                    #vis struct #index_ident;

                    impl ::deli::ModelIndex for #index_ident {
                        const NAME: &'static str = #index_name;

                        type Model = #index_model;

                        type Key = #index_ty;

                        fn index_builder() -> ::deli::reexports::idb::builder::IndexBuilder {
                            ::deli::reexports::idb::builder::IndexBuilder::new(
                                ::std::string::ToString::to_string(<Self as ::deli::ModelIndex>::NAME),
                                ::deli::reexports::idb::KeyPath::new_single( #key ),
                            )
                            .multi_entry(true)
                        }
                    }
                }
            }
            IndexContext::Composite {
                vis,
                keys,
                index_ident,
                index_name,
                index_model,
                index_tys,
                ..
            } => {
                quote! {
                    #vis struct #index_ident;

                    impl ::deli::ModelIndex for #index_ident {
                        const NAME: &'static str = #index_name;

                        type Model = #index_model;

                        type Key = ( #(#index_tys),* );

                        fn index_builder() -> ::deli::reexports::idb::builder::IndexBuilder {
                            ::deli::reexports::idb::builder::IndexBuilder::new(
                                ::std::string::ToString::to_string(<Self as ::deli::ModelIndex>::NAME),
                                ::deli::reexports::idb::KeyPath::new_array([ #(#keys),* ]),
                            )
                        }
                    }
                }
            }
            IndexContext::CompositeUnique {
                vis,
                keys,
                index_ident,
                index_name,
                index_model,
                index_tys,
                ..
            } => {
                quote! {
                    #vis struct #index_ident;

                    impl ::deli::ModelIndex for #index_ident {
                        const NAME: &'static str = #index_name;

                        type Model = #index_model;

                        type Key = ( #(#index_tys),* );

                        fn index_builder() -> ::deli::reexports::idb::builder::IndexBuilder {
                            ::deli::reexports::idb::builder::IndexBuilder::new(
                                ::std::string::ToString::to_string(<Self as ::deli::ModelIndex>::NAME),
                                ::deli::reexports::idb::KeyPath::new_array([ #(#keys),* ]),
                            )
                            .unique(true)
                        }
                    }
                }
            }
            IndexContext::CompositeMultiEntry {
                vis,
                keys,
                index_ident,
                index_name,
                index_model,
                index_tys,
                ..
            } => {
                quote! {
                    #vis struct #index_ident;

                    impl ::deli::ModelIndex for #index_ident {
                        const NAME: &'static str = #index_name;

                        type Model = #index_model;

                        type Key = ( #(#index_tys),* );

                        fn index_builder() -> ::deli::reexports::idb::builder::IndexBuilder {
                            ::deli::reexports::idb::builder::IndexBuilder::new(
                                ::std::string::ToString::to_string(<Self as ::deli::ModelIndex>::NAME),
                                ::deli::reexports::idb::KeyPath::new_array([ #(#keys),* ]),
                            )
                            .multi_entry(true)
                        }
                    }
                }
            }
        }
    }
}

fn get_indexes(model: &Model) -> Result<Vec<IndexContext<'_>>, Error> {
    let mut accumulator = Accumulator::default();
    let mut indexes = Vec::new();

    for field in model.fields() {
        match get_single_index_for_field(model, field) {
            Ok(Some(index)) => indexes.push(index),
            Ok(None) => {}
            Err(err) => accumulator.push(err),
        }
    }

    for meta in model.index.iter() {
        match get_composite_index_for_meta(model, meta) {
            Ok(index) => indexes.push(index),
            Err(err) => accumulator.push(err),
        }
    }

    for meta in model.unique.iter() {
        match get_composite_unique_index_for_meta(model, meta) {
            Ok(index) => indexes.push(index),
            Err(err) => accumulator.push(err),
        }
    }

    for meta in model.multi_entry.iter() {
        match get_composite_multi_entry_index_for_meta(model, meta) {
            Ok(index) => indexes.push(index),
            Err(err) => accumulator.push(err),
        }
    }

    accumulator.finish()?;

    Ok(indexes)
}

fn get_single_index_for_field<'a>(
    model: &'a Model,
    field: &'a ModelField,
) -> Result<Option<IndexContext<'a>>, Error> {
    if !field.is_index() {
        return Ok(None);
    }

    if [
        field.index.is_some(),
        field.unique.is_some(),
        field.multi_entry.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count()
        > 1
    {
        return Err(
            Error::custom("Field can only one of index, unique, or multi_entry attribute")
                .with_span(&field.ident),
        );
    }

    let vis = &model.vis;
    let key = field.get_name_str();
    let index_model = &model.ident;
    let index_ty = &field.ty;

    if let Some(index_meta) = &field.index {
        let (index_ident, index_name) = match index_meta {
            Override::Inherit => (
                Ident::new(
                    &format!(
                        "{}{}Index",
                        model.ident,
                        RenameRule::PascalCase.apply_to_field(field.ident().to_string())
                    ),
                    field.ident().span(),
                ),
                Cow::Owned(LitStr::new(
                    &format!(
                        "{}_{}_index",
                        RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                        field.ident()
                    ),
                    field.ident().span(),
                )),
            ),
            Override::Explicit(index_meta) => {
                let index_ident = match &index_meta.struct_name {
                    None => Ident::new(
                        &format!(
                            "{}{}Index",
                            model.ident,
                            RenameRule::PascalCase.apply_to_field(field.ident().to_string())
                        ),
                        field.ident().span(),
                    ),
                    Some(struct_name) => Ident::new(&struct_name.value(), struct_name.span()),
                };

                let index_name = match &index_meta.name {
                    None => Cow::Owned(LitStr::new(
                        &format!(
                            "{}_{}_index",
                            RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                            field.ident()
                        ),
                        field.ident().span(),
                    )),
                    Some(name) => Cow::Borrowed(name),
                };

                (index_ident, index_name)
            }
        };

        let by_fn_ident = Ident::new(&format!("by_{}", field.ident()), field.ident().span());

        Ok(Some(IndexContext::Single {
            vis,
            key,
            index_ident,
            index_name,
            index_model,
            index_ty,
            by_fn_ident,
        }))
    } else if let Some(unique_meta) = &field.unique {
        let (index_ident, index_name) = match unique_meta {
            Override::Inherit => (
                Ident::new(
                    &format!(
                        "{}{}UniqueIndex",
                        model.ident,
                        RenameRule::PascalCase.apply_to_field(field.ident().to_string())
                    ),
                    field.ident().span(),
                ),
                Cow::Owned(LitStr::new(
                    &format!(
                        "{}_{}_unique_index",
                        RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                        field.ident()
                    ),
                    field.ident().span(),
                )),
            ),
            Override::Explicit(index_meta) => {
                let index_ident = match &index_meta.struct_name {
                    None => Ident::new(
                        &format!(
                            "{}{}UniqueIndex",
                            model.ident,
                            RenameRule::PascalCase.apply_to_field(field.ident().to_string())
                        ),
                        field.ident().span(),
                    ),
                    Some(struct_name) => Ident::new(&struct_name.value(), struct_name.span()),
                };

                let index_name = match &index_meta.name {
                    None => Cow::Owned(LitStr::new(
                        &format!(
                            "{}_{}_unique_index",
                            RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                            field.ident()
                        ),
                        field.ident().span(),
                    )),
                    Some(name) => Cow::Borrowed(name),
                };

                (index_ident, index_name)
            }
        };

        let by_fn_ident = Ident::new(
            &format!("by_{}_unique", field.ident()),
            field.ident().span(),
        );

        Ok(Some(IndexContext::SingleUnique {
            vis,
            key,
            index_ident,
            index_name,
            index_model,
            index_ty,
            by_fn_ident,
        }))
    } else if let Some(multi_entry_meta) = &field.multi_entry {
        let (index_ident, index_name) = match multi_entry_meta {
            Override::Inherit => (
                Ident::new(
                    &format!(
                        "{}{}MultiEntryIndex",
                        model.ident,
                        RenameRule::PascalCase.apply_to_field(field.ident().to_string())
                    ),
                    field.ident().span(),
                ),
                Cow::Owned(LitStr::new(
                    &format!(
                        "{}_{}_multi_entry_index",
                        RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                        field.ident()
                    ),
                    field.ident().span(),
                )),
            ),
            Override::Explicit(index_meta) => {
                let index_ident = match &index_meta.struct_name {
                    None => Ident::new(
                        &format!(
                            "{}{}MultiEntryIndex",
                            model.ident,
                            RenameRule::PascalCase.apply_to_field(field.ident().to_string())
                        ),
                        field.ident().span(),
                    ),
                    Some(struct_name) => Ident::new(&struct_name.value(), struct_name.span()),
                };

                let index_name = match &index_meta.name {
                    None => Cow::Owned(LitStr::new(
                        &format!(
                            "{}_{}_multi_entry_index",
                            RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                            field.ident()
                        ),
                        field.ident().span(),
                    )),
                    Some(name) => Cow::Borrowed(name),
                };

                (index_ident, index_name)
            }
        };

        let by_fn_ident = Ident::new(
            &format!("by_{}_multi_entry", field.ident()),
            field.ident().span(),
        );

        Ok(Some(IndexContext::SingleMultiEntry {
            vis,
            key,
            index_ident,
            index_name,
            index_model,
            index_ty,
            by_fn_ident,
        }))
    } else {
        unreachable!()
    }
}

fn get_composite_index_for_meta<'a>(
    model: &'a Model,
    meta: &'a ModelIndexMeta,
) -> Result<IndexContext<'a>, Error> {
    let fields = model.get_fields_from_path_list(&meta.fields)?;

    let vis = &model.vis;
    let keys = fields
        .iter()
        .map(|field| field.get_name_str())
        .collect::<Vec<_>>();
    let index_model = &model.ident;
    let index_tys = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();

    let index_name = match &meta.name {
        Some(name) => Cow::Borrowed(name),
        None => Cow::Owned(LitStr::new(
            &format!(
                "{}_{}_composite_index",
                RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                fields
                    .iter()
                    .map(|field| field.ident().to_string())
                    .collect::<Vec<_>>()
                    .join("_")
            ),
            model.ident.span(),
        )),
    };

    let index_ident: Ident = match &meta.struct_name {
        Some(struct_name) => Ident::new(&struct_name.value(), struct_name.span()),
        None => Ident::new(
            &format!(
                "{}{}CompositeIndex",
                model.ident,
                fields
                    .iter()
                    .map(|field| RenameRule::PascalCase.apply_to_field(field.ident().to_string()))
                    .collect::<Vec<_>>()
                    .join(""),
            ),
            model.ident.span(),
        ),
    };

    let by_fn_ident = Ident::new(
        &format!(
            "by_{}_composite",
            fields
                .iter()
                .map(|field| field.ident().to_string())
                .collect::<Vec<_>>()
                .join("_")
        ),
        model.ident.span(),
    );

    Ok(IndexContext::Composite {
        vis,
        keys,
        index_ident,
        index_name,
        index_model,
        index_tys,
        by_fn_ident,
    })
}

fn get_composite_unique_index_for_meta<'a>(
    model: &'a Model,
    meta: &'a ModelIndexMeta,
) -> Result<IndexContext<'a>, Error> {
    let fields = model.get_fields_from_path_list(&meta.fields)?;

    let vis = &model.vis;
    let keys = fields
        .iter()
        .map(|field| field.get_name_str())
        .collect::<Vec<_>>();
    let index_model = &model.ident;
    let index_tys = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();

    let index_name = match &meta.name {
        Some(name) => Cow::Borrowed(name),
        None => Cow::Owned(LitStr::new(
            &format!(
                "{}_{}_composite_unique_index",
                RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                fields
                    .iter()
                    .map(|field| field.ident().to_string())
                    .collect::<Vec<_>>()
                    .join("_")
            ),
            model.ident.span(),
        )),
    };

    let index_ident: Ident = match &meta.struct_name {
        Some(struct_name) => Ident::new(&struct_name.value(), struct_name.span()),
        None => Ident::new(
            &format!(
                "{}{}CompositeUniqueIndex",
                model.ident,
                fields
                    .iter()
                    .map(|field| RenameRule::PascalCase.apply_to_field(field.ident().to_string()))
                    .collect::<Vec<_>>()
                    .join(""),
            ),
            model.ident.span(),
        ),
    };

    let by_fn_ident = Ident::new(
        &format!(
            "by_{}_composite_unique",
            fields
                .iter()
                .map(|field| field.ident().to_string())
                .collect::<Vec<_>>()
                .join("_")
        ),
        model.ident.span(),
    );

    Ok(IndexContext::CompositeUnique {
        vis,
        keys,
        index_ident,
        index_name,
        index_model,
        index_tys,
        by_fn_ident,
    })
}

fn get_composite_multi_entry_index_for_meta<'a>(
    model: &'a Model,
    meta: &'a ModelIndexMeta,
) -> Result<IndexContext<'a>, Error> {
    let fields = model.get_fields_from_path_list(&meta.fields)?;

    let vis = &model.vis;
    let keys = fields
        .iter()
        .map(|field| field.get_name_str())
        .collect::<Vec<_>>();
    let index_model = &model.ident;
    let index_tys = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();

    let index_name = match &meta.name {
        Some(name) => Cow::Borrowed(name),
        None => Cow::Owned(LitStr::new(
            &format!(
                "{}_{}_composite_multi_entry_index",
                RenameRule::SnakeCase.apply_to_variant(model.ident.to_string()),
                fields
                    .iter()
                    .map(|field| field.ident().to_string())
                    .collect::<Vec<_>>()
                    .join("_")
            ),
            model.ident.span(),
        )),
    };

    let index_ident: Ident = match &meta.struct_name {
        Some(struct_name) => Ident::new(&struct_name.value(), struct_name.span()),
        None => Ident::new(
            &format!(
                "{}{}CompositeMultiEntryIndex",
                model.ident,
                fields
                    .iter()
                    .map(|field| RenameRule::PascalCase.apply_to_field(field.ident().to_string()))
                    .collect::<Vec<_>>()
                    .join(""),
            ),
            model.ident.span(),
        ),
    };

    let by_fn_ident = Ident::new(
        &format!(
            "by_{}_composite_multi_entry",
            fields
                .iter()
                .map(|field| field.ident().to_string())
                .collect::<Vec<_>>()
                .join("_")
        ),
        model.ident.span(),
    );

    Ok(IndexContext::CompositeMultiEntry {
        vis,
        keys,
        index_ident,
        index_name,
        index_model,
        index_tys,
        by_fn_ident,
    })
}
