use idb::builder::IndexBuilder;
use serde::{de::DeserializeOwned, Serialize};

use crate::model::Model;

/// Trait for defining indexes in an indexed db database model
pub trait ModelIndex {
    /// Name of the index
    const NAME: &'static str;

    /// The model type associated with this index
    type Model: Model;

    /// Type of key for the index
    type Key: Serialize + DeserializeOwned;

    /// Returns the index builder for the index
    #[doc(hidden)]
    fn index_builder() -> IndexBuilder;
}
