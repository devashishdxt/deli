use std::borrow::Cow;

use darling::{ast::Data, error::Accumulator, util::PathList, Error, FromDeriveInput};
use ident_case::RenameRule;
use syn::{Attribute, Generics, Ident, LitStr, Visibility};

use crate::{index_meta::ModelIndexMeta, model_field::ModelField};

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(deli),
    forward_attrs(allow, doc, serde),
    supports(struct_named)
)]
pub struct Model {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub name: Option<LitStr>,
    pub object_store_name: Option<LitStr>,
    pub object_store_struct: Option<LitStr>,
    pub add_struct_name: Option<LitStr>,
    pub key: Option<PathList>,
    #[darling(multiple)]
    pub index: Vec<ModelIndexMeta>,
    #[darling(multiple)]
    pub unique: Vec<ModelIndexMeta>,
    #[darling(multiple)]
    pub multi_entry: Vec<ModelIndexMeta>,
    pub data: Data<(), ModelField>,
    pub attrs: Vec<Attribute>,
}

impl Model {
    pub fn validate_no_generic(&self) -> Result<(), Error> {
        if !self.generics.params.is_empty() {
            Err(
                Error::custom("Generic type is not supported by `deli::Model` derive macro")
                    .with_span(&self.ident.span()),
            )
        } else {
            Ok(())
        }
    }

    pub fn get_name_str(&self) -> Cow<'_, LitStr> {
        match &self.name {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(LitStr::new(
                &RenameRule::SnakeCase.apply_to_variant(self.ident.to_string()),
                self.ident.span(),
            )),
        }
    }

    pub fn fields(&self) -> &[ModelField] {
        match self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(ref data) => data.fields.as_slice(),
        }
    }

    pub fn get_fields_from_path_list(
        &self,
        path_list: &PathList,
    ) -> Result<Vec<&ModelField>, Error> {
        let mut accumulator = Accumulator::default();
        let mut fields = Vec::new();

        for path in path_list.iter() {
            let ident = path.get_ident();

            match ident {
                None => {
                    accumulator.push(Error::custom("This must be an identifier").with_span(&path));
                }
                Some(ident) => {
                    let field = self
                        .fields()
                        .iter()
                        .find(|field| field.ident.as_ref().unwrap() == ident);

                    match field {
                        None => {
                            accumulator.push(
                                Error::custom("Field not found in the model").with_span(&ident),
                            );
                        }
                        Some(field) => fields.push(field),
                    }
                }
            }
        }

        accumulator.finish()?;

        Ok(fields)
    }
}
