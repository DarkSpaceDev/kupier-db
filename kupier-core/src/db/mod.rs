use std::{pin::Pin, sync::Arc};

use bson::{doc, Bson, Document};
use rocksdb::{OptimisticTransactionDB, TransactionDB, DB};
use uuid::Uuid;

pub mod information_schema;

#[derive(Clone)]
pub struct DbEngine {
    kv_storage: Pin<Arc<OptimisticTransactionDB>>,
    indexes: Vec<String>,
}

// #[derive(Clone)]
// pub struct Datastore {
//     db: Pin<Arc<OptimisticTransactionDB>>,
// }

impl DbEngine {
    pub fn new(db_path: &str, db_name: &str) -> DbEngine {
        let mut options = rocksdb::Options::default();
        options.set_compression_type(rocksdb::DBCompressionType::None);
        options.set_error_if_exists(false);
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let mut this_db_path: String = db_path.to_owned();
        this_db_path.push_str("\\");
        this_db_path.push_str(db_name);

        let mut indexes = DB::list_cf(&options, this_db_path.clone().as_str()).unwrap_or(vec![]);
        let cfs = indexes.clone();

        let default_index = indexes.iter().position(|x| *x == "default");

        if !default_index.is_none() {
            indexes.remove(default_index.unwrap());
        }

        DbEngine {
            kv_storage: Arc::pin(
                OptimisticTransactionDB::open_cf(&options, this_db_path, cfs).unwrap(),
            ),
            indexes: indexes,
        }
    }

    pub fn index_exists(&mut self, idx_name: &str) -> bool {
        self.kv_storage.cf_handle(idx_name).is_some()
    }

    pub fn add_index(&mut self, idx_name: &str) {
        if !self.index_exists(idx_name) {
            let options = rocksdb::Options::default();
            self.kv_storage.create_cf(idx_name, &options).unwrap();
        }
    }

    pub fn drop_index(&mut self, idx_name: &str) {
        if self.index_exists(idx_name) {
            self.kv_storage.drop_cf(idx_name).unwrap();
        }
    }

    pub fn get(&mut self, id: &str) -> Option<Document> {
        let value = self.kv_storage.get(id.as_bytes()).unwrap();

        if value.is_none() {
            return Option::None;
        }

        let bytes = value.unwrap();
        let doc = Document::from_reader(&mut bytes.as_slice()).unwrap();

        return Some(doc);
    }

    fn update_index(&mut self, key: Vec<u8>, field: &str, value: bson::Bson) {
        // Validate type being used for indexing
        match value.element_type() {
            bson::spec::ElementType::ObjectId
            | bson::spec::ElementType::Binary
            | bson::spec::ElementType::Boolean
            | bson::spec::ElementType::Decimal128
            | bson::spec::ElementType::DateTime
            | bson::spec::ElementType::Double
            | bson::spec::ElementType::Int32
            | bson::spec::ElementType::Int64
            | bson::spec::ElementType::Null
            | bson::spec::ElementType::String
            | bson::spec::ElementType::Timestamp => {
                let cf = self.kv_storage.cf_handle(field).unwrap();

                self.kv_storage
                    .put_cf(&cf, &key, bson::to_vec(&doc! { "0": value }).unwrap())
                    .unwrap();
            }
            _ => {
                // This type isn't supported - skip it.
            }
        }
    }

    // There's a lot going on with identifiers and it should be fixed
    // as it could cause all sorts of funny stuff. This needs to be ACID
    // thread safe, etc.
    pub fn insert(&mut self, document: Document) -> Option<String> {
        let mut working_document = document.clone();
        let id = working_document.get_str("id");
        let mut record_id = String::from("");
        let mut generate_id = false;

        if id.is_err() {
            let err: bson::document::ValueAccessError = id.err().unwrap();
            match err {
                bson::document::ValueAccessError::NotPresent => {
                    generate_id = true;
                }
                bson::document::ValueAccessError::UnexpectedType => {
                    return Option::None;
                }
                _ => todo!(),
            }
        } else {
            record_id = String::from(id.ok().unwrap());
        }

        // TODO: Implement a retry on collision
        if generate_id {
            record_id = Uuid::new_v4().to_string();
        }

        let mut buf = Vec::new();
        working_document.to_writer(&mut buf).unwrap();

        if self.kv_storage.get(record_id.as_bytes()).unwrap().is_none() {
            working_document.insert("id", Bson::String(record_id.clone()));
            for key in working_document.keys() {
                let value = working_document.get(key.as_str()).unwrap();
                let id_bytes = record_id.clone();

                if self.indexes.contains(key) {
                    self.update_index(id_bytes.into_bytes(), key.as_str(), value.clone());
                }
            }

            self.kv_storage.put(record_id.as_bytes(), buf).unwrap();
        }

        return Some(record_id);
    }
}
