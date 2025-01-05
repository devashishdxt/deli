use crate::{database::Database, error::Error, model::Model};

/// A builder for [`Database`]
#[derive(Debug)]
pub struct DatabaseBuilder {
    builder: idb::builder::DatabaseBuilder,
}

impl DatabaseBuilder {
    /// Creates a new instance of [`DatabaseBuilder`]
    pub fn new(name: &str) -> Self {
        Self {
            builder: idb::builder::DatabaseBuilder::new(name),
        }
    }

    /// Sets the version of the database
    pub fn version(mut self, version: u32) -> Self {
        self.builder = self.builder.version(version);
        self
    }

    /// Adds a model to the database
    pub fn add_model<M>(mut self) -> Self
    where
        M: Model,
    {
        self.builder = self.builder.add_object_store(M::object_store_builder());
        self
    }

    /// Builds the [`Database`] instance
    pub async fn build(self) -> Result<Database, Error> {
        self.builder
            .build()
            .await
            .map(Database::new)
            .map_err(Into::into)
    }
}
