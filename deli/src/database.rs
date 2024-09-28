use std::mem::take;

use idb::{event::VersionChangeEvent, Database as IdbDatabase, Factory};

use crate::{Error, Model, TransactionBuilder};

/// [`Database`] provides connection to an indexed db database
#[derive(Debug)]
pub struct Database {
    database: IdbDatabase,
}

impl Database {
    /// Creates a new instance of [`Database`]
    pub async fn new(name: String, version: u32) -> Result<Self, Error> {
        Self::builder(name).version(version).build().await
    }

    /// Returns a builder for [`Database`]
    pub fn builder(name: String) -> DatabaseBuilder {
        DatabaseBuilder::new(name)
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
        factory.delete(name)?.await.map_err(Into::into)
    }

    /// Returns the inner [`IdbDatabase`] handle
    pub(crate) fn database(&self) -> &IdbDatabase {
        &self.database
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.close()
    }
}

/// A builder for [`Database`]
pub struct DatabaseBuilder {
    name: String,
    version: Option<u32>,
    models: Vec<Box<dyn Fn(VersionChangeEvent) + 'static>>,
}

impl DatabaseBuilder {
    /// Creates a new instance of [`DatabaseBuilder`]
    pub fn new(name: String) -> Self {
        DatabaseBuilder {
            name,
            version: None,
            models: Vec::new(),
        }
    }

    /// Set database version
    pub fn version(&mut self, version: u32) -> &mut Self {
        self.version = Some(version);
        self
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
    pub async fn build(&mut self) -> Result<Database, Error> {
        let factory = Factory::new()?;
        let mut open_request = factory.open(&self.name, self.version)?;

        let models = take(&mut self.models);

        open_request.on_upgrade_needed(move |event| {
            for model in models.into_iter() {
                model(event.clone());
            }
        });

        let mut database = open_request.await?;
        database.on_version_change(|version_change_event| {
            version_change_event.current_target().map(|target| {
                target.try_into().map(|database: IdbDatabase| {
                    database.close();
                })
            });
        });

        Ok(Database { database })
    }
}
