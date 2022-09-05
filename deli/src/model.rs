use idb::VersionChangeEvent;
use serde::{de::DeserializeOwned, Serialize};

/// Trait for defining object stores in an indexed db database
pub trait Model: DeserializeOwned {
    /// Name of the object store
    const NAME: &'static str;

    /// Type of key for the model
    type Key: Serialize + DeserializeOwned;

    /// Upgrade handler for the object store
    fn handle_upgrade(event: VersionChangeEvent);
}
