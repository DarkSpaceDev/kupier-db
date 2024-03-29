//--------------------------------------------------------------------------
// (C) Copyright Travis Sharp <travis@darkspace.dev>.  All rights reserved.
//--------------------------------------------------------------------------

use super::kv::Key;
use super::kv::Val;
use crate::error::Error;
use futures::lock::Mutex;
use rocksdb::{
    OptimisticTransactionDB, OptimisticTransactionOptions, ReadOptions, WriteOptions, DB,
};
use std::ops::Neg;
use std::ops::Range;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub struct Datastore {
    db: Pin<Arc<OptimisticTransactionDB>>,
    indexes: Vec<String>,
}

pub struct Transaction {
    // The datastore transaction
    txn: Arc<Mutex<Option<rocksdb::Transaction<'static, OptimisticTransactionDB>>>>,

    // Has the transaction completed.
    completed: bool,

    // Is the transaction ReadWrite, true, or ReadOnly, false.
    rw: bool,

    // The read options regarding the transaction Snapshot
    _snapshot_read_options: ReadOptions,

    // the above 'static transaction points here in order to keep
    // the memory alive - to make sure that this is dropped last,
    // it must be declared last
    _db: Pin<Arc<OptimisticTransactionDB>>,

    // This is the datastore that the db is tied to
    _ds: Pin<Arc<Datastore>>,
}

impl Datastore {
    /// Open a new database
    pub async fn new(path: &str) -> Result<Datastore, Error> {
        let mut options = rocksdb::Options::default();
        options.set_compression_type(rocksdb::DBCompressionType::None);
        options.set_error_if_exists(false);
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        // Disable WAL for specific write operations
        // options.set_atomic_flush(true);
        // let mut write_options = WriteOptions::default();
        // write_options.disable_wal(true);

        let mut indexes = DB::list_cf(&options, path.clone()).unwrap_or(vec![]);
        let cfs = indexes.clone();

        let default_index = indexes.iter().position(|x| *x == "default");

        if !default_index.is_none() {
            indexes.remove(default_index.unwrap());
        }

        Ok(Datastore {
            db: Arc::pin(OptimisticTransactionDB::open_cf(&options, path, cfs)?),
            indexes: indexes,
        })
    }

    fn index_exists(&self, idx_name: &str) -> bool {
        self.db.cf_handle(idx_name).is_some()
    }

    pub fn add_index(&mut self, idx_name: &str) {
        if !self.index_exists(idx_name) {
            let options = rocksdb::Options::default();
            self.indexes.push(idx_name.to_owned());
            self.db.create_cf(idx_name, &options).unwrap();
        }
    }

    pub fn drop_index(&mut self, idx_name: &str) {
        if self.index_exists(idx_name) {
            self.db.drop_cf(idx_name).unwrap();

            let index_to_remove = self.indexes.iter().position(|x| *x == idx_name);

            if !index_to_remove.is_none() {
                self.indexes.remove(index_to_remove.unwrap());
            }
        }
    }

    /// Start a new transaction
    pub async fn transaction(&self, write: bool) -> Result<Transaction, Error> {
        // snapshot options
        let mut transaction_options = OptimisticTransactionOptions::default();
        transaction_options.set_snapshot(true);

        let write_options = WriteOptions::default();
        // When atomic flush is enabled, disable wal
        // write_options.disable_wal(true);

        // Create a new transaction
        let txn = self
            .db
            .transaction_opt(&write_options, &transaction_options);

        // The database reference must always outlive
        // the transaction. If it doesn't then this
        // is undefined behaviour. This unsafe block
        // ensures that the transaction reference is
        // static, but will cause a crash if the
        // datastore is dropped prematurely.
        let txn = unsafe {
            std::mem::transmute::<
                rocksdb::Transaction<'_, OptimisticTransactionDB>,
                rocksdb::Transaction<'static, OptimisticTransactionDB>,
            >(txn)
        };

        let mut snapshot_read_options = ReadOptions::default();
        snapshot_read_options.set_snapshot(&txn.snapshot());

        // Return the transaction
        Ok(Transaction {
            completed: false,
            rw: write,
            txn: Arc::new(Mutex::new(Some(txn))),
            _snapshot_read_options: snapshot_read_options,
            _db: self.db.clone(),
            _ds: Arc::pin(self.clone()),
        })
    }
}

impl Transaction {
    /** Check to see if txn is completed, this could mean either commited or rolled back */
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /** Discard all transaction operations */
    pub async fn rollback(&mut self) -> Result<(), Error> {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Mark this transaction as completed
        self.completed = true;

        // Rollback this transaction
        match self.txn.lock().await.take() {
            Some(txn) => txn.rollback()?,
            None => unreachable!(),
        };

        // Continue
        Ok(())
    }

    pub async fn update_index<K, V>(&mut self, idx: &str, key: K, val: V) -> Result<(), Error>
    where
        K: Into<Key>,
        V: Into<Val>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Check to see if transaction is writable
        if !self.rw {
            return Err(Error::TxReadonly);
        }

        if !self._ds.index_exists(idx) {
            return Err(Error::Tx(format!("`{}` index does not exist.", idx)));
        }

        let cf = self._db.cf_handle(idx).unwrap();

        // Get the arguments
        let key = key.into();
        let val = val.into();

        self.txn
            .lock()
            .await
            .as_ref()
            .unwrap()
            .put_cf(&cf, key, val)?;

        // Continue
        Ok(())
    }

    /** Commit transaction */
    pub async fn commit(&mut self) -> Result<(), Error> {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Check to see if transaction is writable
        if !self.rw {
            return Err(Error::TxReadonly);
        }

        // Mark this transaction as done
        self.completed = true;

        // Cancel this transaction
        match self.txn.lock().await.take() {
            Some(txn) => txn.commit()?,
            None => unreachable!(),
        };

        // Continue
        Ok(())
    }

    /// Check if a key exists
    pub async fn key_exists<K>(&mut self, key: K) -> Result<bool, Error>
    where
        K: Into<Key>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Check the key
        let res = self
            .txn
            .lock()
            .await
            .as_ref()
            .unwrap()
            .get_opt(key.into(), &self._snapshot_read_options)?
            .is_some();

        // Return result
        Ok(res)
    }

    /// Fetch a key from the database
    pub async fn get<K>(&mut self, key: K) -> Result<Option<Val>, Error>
    where
        K: Into<Key>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Get the key
        let res = self
            .txn
            .lock()
            .await
            .as_ref()
            .unwrap()
            .get_opt(key.into(), &self._snapshot_read_options)?;

        // Return result
        Ok(res)
    }

    /// Insert or update a key in the database
    pub async fn upsert<K, V>(&mut self, key: K, val: V) -> Result<(), Error>
    where
        K: Into<Key>,
        V: Into<Val>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Check to see if transaction is writable
        if !self.rw {
            return Err(Error::TxReadonly);
        }

        // Set the key
        self.txn
            .lock()
            .await
            .as_ref()
            .unwrap()
            .put(key.into(), val.into())?;

        // Return result
        Ok(())
    }

    /// Insert a key if it doesn't exist in the database
    pub async fn insert<K, V>(&mut self, key: K, val: V) -> Result<(), Error>
    where
        K: Into<Key>,
        V: Into<Val>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Check to see if transaction is writable
        if !self.rw {
            return Err(Error::TxReadonly);
        }

        // Get the transaction
        let txn = self.txn.lock().await;
        let txn = txn.as_ref().unwrap();

        // Get the arguments
        let key = key.into();
        let val = val.into();

        // Set the key if empty
        match txn.get_opt(&key, &self._snapshot_read_options)? {
            None => txn.put(key, val)?,
            _ => return Err(Error::TxKeyAlreadyExists),
        };

        // Return result
        Ok(())
    }

    /// Insert a key if it doesn't exist in the database
    pub async fn insert_checked<K, V>(
        &mut self,
        key: K,
        val: V,
        chk: Option<V>,
    ) -> Result<(), Error>
    where
        K: Into<Key>,
        V: Into<Val>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Check to see if transaction is writable
        if !self.rw {
            return Err(Error::TxReadonly);
        }

        // Get the transaction
        let txn = self.txn.lock().await;
        let txn = txn.as_ref().unwrap();

        // Get the arguments
        let key = key.into();
        let val = val.into();
        let chk = chk.map(Into::into);

        // Set the key if valid
        match (txn.get_opt(&key, &self._snapshot_read_options)?, chk) {
            (Some(v), Some(w)) if v == w => txn.put(key, val)?,
            (None, None) => txn.put(key, val)?,
            _ => return Err(Error::TxConditionNotMet),
        };

        // Return result
        Ok(())
    }

    /// Delete a key
    pub async fn delete<K>(&mut self, key: K) -> Result<(), Error>
    where
        K: Into<Key>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Check to see if transaction is writable
        if !self.rw {
            return Err(Error::TxReadonly);
        }

        // Remove the key
        self.txn.lock().await.as_ref().unwrap().delete(key.into())?;

        // Return result
        Ok(())
    }

    pub async fn scan<K>(
        &mut self,
        limit: u32,
        after: Option<Vec<u8>>,
    ) -> Result<Vec<(Key, Val)>, Error>
    where
        K: Into<Key>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Get the transaction
        let txn = self.txn.lock().await;
        let txn = txn.as_ref().unwrap();

        // Create result set
        let mut res = vec![];

        // Set the ReadOptions with the snapshot
        let mut read_options = ReadOptions::default();
        read_options.set_snapshot(&txn.snapshot());

        // Create the iterator
        let mut iterator = txn.raw_iterator_opt(read_options);

        // Prime the iterator
        if after == Option::None {
            iterator.seek_to_first();
        } else {
            iterator.seek(after.unwrap());

            // Advance past the key ...
            if iterator.valid() {
                iterator.next();
            }
        }

        // Scan the keys in the iterator
        while iterator.valid() {
            // Check the scan limit
            if res.len() < limit as usize {
                // Get the key and value
                let (k, v) = (iterator.key(), iterator.value());

                // Check the key and value
                if let (Some(k), Some(v)) = (k, v) {
                    res.push((k.to_vec(), v.to_vec()));
                    iterator.next();
                    continue;
                }
            }

            // Exit
            break;
        }

        // Return result
        Ok(res)
    }

    pub async fn scan_range<K>(&mut self, rng: Range<K>, limit: u32) -> Result<Vec<Key>, Error>
    where
        K: Into<Key>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Get the transaction
        let txn = self.txn.lock().await;
        let txn = txn.as_ref().unwrap();

        // Convert the range to bytes
        let rng: Range<Key> = Range {
            start: rng.start.into(),
            end: rng.end.into(),
        };

        // Create result set
        let mut res = vec![];

        // Set the key range
        let beg = rng.start.as_slice();
        let end = rng.end.as_slice();

        // Set the ReadOptions with the snapshot
        let mut read_options = ReadOptions::default();
        read_options.set_snapshot(&txn.snapshot());

        // Create the iterator
        let mut iter = txn.raw_iterator_opt(read_options);

        // Seek to the start key
        iter.seek(&rng.start);

        // Scan the keys in the iterator
        while iter.valid() {
            // Check the scan limit
            if res.len() < limit as usize {
                // Get the key and value
                let k = iter.key();

                // Check the key and value
                if let Some(k) = k {
                    if k >= beg && k < end {
                        res.push(k.to_vec());
                        iter.next();
                        continue;
                    }
                }
            }

            // Exit
            break;
        }

        // Return result
        Ok(res)
    }

    /// Retrieve a range of keys from the databases based on a certain collection prefix
    pub async fn scan_collection_index<K>(
        &mut self,
        collection_prefix: Vec<u8>,
        column: &str,
        limit: u32,
        after: Option<Vec<u8>>,
    ) -> Result<Vec<(Key, Val)>, Error>
    where
        K: Into<Key>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Get the transaction
        let txn = self.txn.lock().await;
        let txn = txn.as_ref().unwrap();

        // Create result set
        let mut res = vec![];

        // Set the ReadOptions with the snapshot
        let mut read_options = ReadOptions::default();
        read_options.set_snapshot(&txn.snapshot());

        read_options.set_prefix_same_as_start(true);
        read_options.set_iterate_upper_bound(collection_prefix);
        // read_options.set_iterate_range(rocksdb::PrefixRange(collection_prefix));

        // Grab the column family handle ...
        let cf_handle = self
            ._db
            .cf_handle(column)
            .expect("Column family not found.");

        // Create the iterator
        let mut iterator = txn.raw_iterator_cf_opt(&cf_handle, read_options);

        // Prime the iterator
        if after == Option::None {
            iterator.seek_to_first();
        } else {
            iterator.seek(after.unwrap());

            // Advance past the key ...
            if iterator.valid() {
                iterator.next();
            }
        }

        // Scan the keys in the iterator
        while iterator.valid() {
            // Check the scan limit
            if res.len() < limit as usize {
                // Get the key and value
                let (k, v) = (iterator.key(), iterator.value());

                // Check the key and value
                if let (Some(k), Some(v)) = (k, v) {
                    res.push((k.to_vec(), v.to_vec()));
                    iterator.next();
                    continue;
                }
            }

            // Exit
            break;
        }

        // Return result
        Ok(res)
    }

    /// Retrieve a range of keys from the databases based on a certain collection prefix
    pub async fn scan_collection<K>(
        &mut self,
        collection_prefix: Vec<u8>,
        limit: Option<usize>,
        after: Option<Vec<u8>>,
    ) -> Result<Vec<(Key, Val)>, Error>
    where
        K: Into<Key>,
    {
        // Check to see if transaction is closed
        if self.completed {
            return Err(Error::TxFinished);
        }

        // Get the transaction
        let txn = self.txn.lock().await;
        let txn = txn.as_ref().unwrap();

        // Create result set
        let mut res = vec![];

        // Set the ReadOptions with the snapshot
        let mut read_options = ReadOptions::default();
        read_options.set_snapshot(&txn.snapshot());

        read_options.set_prefix_same_as_start(true);
        // read_options.set_iterate_upper_bound(collection_prefix);
        read_options.set_iterate_range(rocksdb::PrefixRange(collection_prefix));

        // Create the iterator
        let mut iterator = txn.raw_iterator_opt(read_options);
        // let iterator = self._db.iterator(IteratorMode::Start);

        // Prime the iterator
        iterator.seek_to_first();

        // Seek to location, if needed
        if after != Option::None {
            iterator.seek(after.unwrap());
        }

        let no_limit = limit.is_none();

        // // Scan the keys in the iterator
        while iterator.valid() {
            // Check the scan limit
            if no_limit || res.len() < limit.unwrap() {
                // Get the key and value
                let (k, v) = (iterator.key(), iterator.value());

                // Check the key and value
                if let (Some(k), Some(v)) = (k, v) {
                    res.push((k.to_vec(), v.to_vec()));
                    iterator.next();
                    continue;
                }
            } else {
                iterator.next();
                iterator.valid();
                println!("DEBUG >> HAS MORE!");
            }

            // Exit
            break;
        }

        // for v in iterator {
        //     let x = v.unwrap();
        //     res.push((x.0.to_vec(), x.1.to_vec()));
        // }

        // Return result
        Ok(res)
    }
}
