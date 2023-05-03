use std::collections::HashMap;
use log::debug;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, RwLock};
use bson::Document;
use crate::error::{Error, Result};
use crate::storage::engine::StorageEngine;

pub mod Config {
    /// Used for all things relevant to data
    pub const DEFAULT_DB_FILE: &str = "kupier.db";

    /// Used for transactions and raft
    pub const DEFAULT_DB_LOG_FILE: &str = "kupier.db.log";
}

pub struct Database {
    storage: StorageEngine
}

impl Database {
    pub fn new() -> Result<Database> {
        debug!("{:?}", "Creating database ...");

        Ok(Database {
            storage: StorageEngine::new(Config::DEFAULT_DB_FILE)
        })
    }

    pub fn save(&mut self) {
        debug!("{:?}", "Saving database ...");
        self.storage.save();
        debug!("{:?}", "Saved.");
    }

    /*
    pub fn load() -> Result<Database> {
        debug!("{:?}", "Loading database ...");

        Ok(Database  {
            storage: StorageEngine::load(Config::DEFAULT_DB_FILE).unwrap()
        })
    }
    */

    pub fn insert(&self, doc: Document) {
        self.storage.insert(doc);
    }
}