use std::fs::OpenOptions;
use tokio::fs::File;
use kupier_core::error::{Result, Error };

pub mod EngineConfig {
    /// Default Block Size in Bytes of 8KB
    pub const DEFAULT_BLOCK_SIZE: u64 = 8192; // 8KB Block Size, 128 per MB
}


pub struct StorageEngine {
    db_file: tokio::fs::File
}

impl StorageEngine {
    pub async fn new(path: &str) -> StorageEngine {
        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path).await;

        let engine = StorageEngine {
            db_file: file.unwrap()
        };

        return engine;
    }

    fn read_block(&self, start_position: u64) -> Result<Vec<u8>> {
        todo!()
    }

    fn write_block(&self, start_position: u64, data: &[u8]) -> Result<()> {
        todo!()
    }

    fn expand(&self, number_of_blocks: u64) -> Result<()> {
        todo!()
    }
}