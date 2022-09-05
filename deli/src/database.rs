use idb::{Database as IdbDatabase, Error, Factory, VersionChangeEvent};

use crate::{Model, TransactionBuilder};

/// [`Database`] provides connection to an indexed db database
#[derive(Debug)]
pub struct Database {
    database: IdbDatabase,
}

impl Database {
    /// Creates a new instance of [`Database`]
    pub async fn new(name: String, version: u32) -> Result<Self, Error> {
        Self::builder(name, version).build().await
    }

    /// Returns a builder for [`Database`]
    pub fn builder(name: String, version: u32) -> DatabaseBuilder {
        DatabaseBuilder::new(name, version)
    }

    /// Returns the name of database
    pub fn name(&self) -> String {
        self.database.name()
    }

    /// Returns the version of database
    pub fn version(&self) -> Result<u32, Error> {
        self.database.version().map_err(Into::into)
    }

    /// Returns a transaction builder for creating transactions on database
    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self)
    }

    /// Closes database connection
    pub fn close(&self) {
        self.database.close();
    }

    /// Deletes a database
    pub async fn delete(name: &str) -> Result<(), Error> {
        let factory = Factory::new()?;
        factory.delete(name).await
    }

    /// Returns the inner [`IdbDatabase`] handle
    pub(crate) fn database(&self) -> &IdbDatabase {
        &self.database
    }
}

/// A builder for [`Database`]
pub struct DatabaseBuilder {
    name: String,
    version: u32,
    models: Vec<Box<dyn Fn(VersionChangeEvent) + 'static>>,
}

impl DatabaseBuilder {
    /// Creates a new instance of [`DatabaseBuilder`]
    pub fn new(name: String, version: u32) -> Self {
        DatabaseBuilder {
            name,
            version,
            models: Vec::new(),
        }
    }

    /// Registers a [`Model`]
    pub fn register_model<M>(&mut self) -> &mut Self
    where
        M: Model,
    {
        self.models.push(Box::new(|event| M::handle_upgrade(event)));
        self
    }

    /// Builds an instance of [`Database`]
    pub async fn build(self) -> Result<Database, Error> {
        let factory = Factory::new()?;
        let mut open_request = factory.open(&self.name, self.version)?;

        open_request.on_upgrade_needed(move |event| {
            for model in self.models.into_iter() {
                model(event.clone());
            }
        });

        let mut database = open_request.into_future().await?;
        database.on_version_change(|database| database.close());

        Ok(Database { database })
    }
}
