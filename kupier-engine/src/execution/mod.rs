use bson::doc;
use bson::oid::ObjectId;

use kupier_core::schema::information_schema;
use kupier_core::{error::Result, storage::rocksdb::Datastore};
use kupier_lang::ast::ScalarValue;
use serde::Deserialize;
use serde_derive::Serialize;
use std::pin::Pin;
use std::sync::Arc;

use crate::plan::CollectionScan;

pub struct Executor {
    _ds: Pin<Arc<Datastore>>,
}

pub struct QueryPlan {
    pub collection: String,
    pub schema: String,
}
pub struct ExecutionContext {
    pub parameters: Vec<(String, ScalarValue)>,
}

#[derive(Serialize, Deserialize)]
pub struct QueryResult {
    pub records: Vec<bson::Bson>,
}

pub struct Record {
    pub key: Vec<u8>,
    pub body: Vec<u8>,
}

impl Executor {
    pub fn new(datastore: Datastore) -> Executor {
        return Executor {
            _ds: Pin::new(Arc::new(datastore)),
        };
    }

    fn generate_collection_prefix(schema: String, collection: String) -> Vec<u8> {
        let mut prefix: Vec<u8> = Vec::new();

        prefix.extend(schema.to_lowercase().into_bytes());
        prefix.extend(information_schema::PADDING);
        prefix.extend(collection.to_lowercase().into_bytes());
        prefix.extend(information_schema::PADDING);

        return prefix;
    }

    fn generate_collection_id_with_bytes(
        schema: String,
        collection: String,
        id: Vec<u8>,
    ) -> Vec<u8> {
        let mut prefix = Executor::generate_collection_prefix(schema, collection);
        prefix.extend(id);

        return prefix;
    }

    // fn generate_collection_id(schema: String, collection: String, id: Uuid) -> Vec<u8> {
    //     let mut prefix = Executor::generate_collection_prefix(schema, collection);
    //     prefix.extend(id.as_bytes());

    //     return prefix;
    // }

    fn generate_collection_id2(schema: String, collection: String, id: ObjectId) -> Vec<u8> {
        let mut prefix = Executor::generate_collection_prefix(schema, collection);
        prefix.extend(id.bytes());

        return prefix;
    }

    pub async fn test_insert(&self, collection: String) -> Result<()> {
        let mut txn = self._ds.transaction(true).await?;
        let id = ObjectId::new();

        let val = doc!["_id": id];

        let val_bytes = bson::to_vec(&val).unwrap();

        txn.insert(
            Self::generate_collection_id2(String::from("default"), collection, id),
            val_bytes,
        )
        .await?;
        txn.commit().await?;

        return Ok(());
    }

    pub async fn test_bulk_insert(&self, collection: String, count: u32) -> Result<()> {
        let mut txn = self._ds.transaction(true).await?;

        for _ in 0..count {
            // let id = Uuid::new_v4();
            // let val = doc![
            //     "_id": bson::Binary {
            //         subtype: BinarySubtype::Uuid,
            //         bytes: id.as_bytes().to_vec()
            //     }
            // ];

            let id = ObjectId::new();
            let val = doc!["_id": id];

            let val_bytes = bson::to_vec(&val).unwrap();

            txn.insert(
                Self::generate_collection_id2(String::from("default"), collection.clone(), id),
                val_bytes,
            )
            .await?;
        }

        txn.commit().await?;

        return Ok(());
    }

    pub async fn create_collection(&self, collection: String) -> Result<()> {
        let mut txn = self._ds.transaction(true).await?;
        let id = ObjectId::new();

        let val = doc![
            "_id": id,
            "schema": information_schema::DEFAULT_SCHEMA,
            "collection": collection,
        ];

        // TODO: Update Unique Clustered Index 'collection'

        let val_bytes = bson::to_vec(&val).unwrap();
        txn.insert(
            Self::generate_collection_id2(
                String::from("information_schema"),
                String::from("table"),
                id,
            ),
            val_bytes,
        )
        .await?;
        txn.commit().await?;

        return Ok(());
    }

    pub async fn execute_select(&self, plan: QueryPlan) -> Result<QueryResult> {
        let prefix = Executor::generate_collection_prefix(plan.schema, plan.collection);

        let mut txn = self._ds.transaction(false).await?;
        let mut last: Option<Vec<u8>> = Option::None;
        let mut records: Vec<bson::Bson> = Vec::new();

        loop {
            let scan_result = txn
                .scan_collection::<Vec<u8>>(prefix.clone(), Some(5000), last.clone())
                .await?;

            let count = scan_result.len();

            last = Option::None;

            for raw_record in scan_result {
                last = Some(raw_record.0);
                let doc: bson::Bson = bson::from_slice(&raw_record.1).ok().unwrap();

                // TODO: Expression Matching / Filtering ...
                records.push(doc);
            }

            if count < 5000 {
                break;
            }
        }

        Ok(QueryResult { records })
    }

    pub async fn execute_collection_scan(
        &self,
        plan: CollectionScan,
    ) -> Result<Vec<bson::Document>> {
        let mut txn = self._ds.transaction(true).await?;
        let prefix =
            Executor::generate_collection_prefix(plan.schema.clone(), plan.collection.clone());
        let mut after = Option::None;
        let limit: usize = 10000;

        let mut results: Vec<bson::Document> = Vec::new();

        loop {
            let records = txn
                .scan_collection::<Vec<u8>>(prefix.clone(), Some(limit), after.clone())
                .await?;

            let loaded_all_data = records.len() < limit;

            // TODO: We don't need to convert the data into something else, we should just be able to scan
            // the bson data directly for better efficiency as per original design of bson.
            let mut iter = records.iter();

            loop {
                match iter.next() {
                    Some(record) => {
                        let document = bson::from_reader(&*record.1).unwrap();
                        results.push(document);
                    }
                    None => {
                        // Iterator has reached the end, break the loop
                        break;
                    }
                }
            }

            if let Some(last_doc) = results.last() {
                let id_raw = last_doc.get("_id").unwrap();
                if let bson::Bson::Binary(binary_data) = id_raw {
                    let id = binary_data.bytes.clone();
                    after = Some(Self::generate_collection_id_with_bytes(
                        plan.schema.clone(),
                        plan.collection.clone(),
                        id,
                    ));
                } else {
                    panic!()
                }
            }

            // page chunk met ..
            if loaded_all_data {
                break;
            }
        }

        txn.commit().await?;

        Ok(results)
    }

    pub async fn execute_count(&self, plan: CollectionScan) -> Result<u64> {
        let mut txn = self._ds.transaction(true).await?;
        let prefix =
            Executor::generate_collection_prefix(plan.schema.clone(), plan.collection.clone());
        let mut after = Option::None;
        let limit: usize = 10000;

        let mut results: u64 = 0;

        loop {
            let records = txn
                .scan_collection::<Vec<u8>>(prefix.clone(), Some(limit), after.clone())
                .await?;

            let loaded_all_data = records.len() < limit;

            if let Some(last_record) = records.last() {
                after = Some(last_record.0.clone());
            }

            results += records.len() as u64;

            // page chunk met ..
            if loaded_all_data {
                break;
            }
        }

        txn.commit().await?;

        Ok(results)
    }
}
