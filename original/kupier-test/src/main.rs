mod calculator;

use kuiperdb_storage::engine::StorageEngine;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{ Arc, RwLock};
use std::fs::File;
use std::io::prelude::Write;
use bson::{doc, Bson, Document};
use chrono::Utc;
use kuiperdb_core::db::Database;

// Logging
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

fn init_db() -> Database {
    let mut db = Database::new().unwrap();
    db.save();
    return db;
}
fn to_bson(value: serde_json::Value) -> Document {
    serde_json::from_value(value).unwrap()
}

#[tokio::main]
async fn main() {
    println!("Running!");
    // main2();
    StorageEngine::new("kuiper.db").await;
}

fn main2 () {
    calculator::test_page_efficiencies();

    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    // First off - everything is stored on the stack ... which will cause things to break quite
    // easily with any significant number of items.

    // Arc
    // RwLock
    // let mut map: BTreeMap<str, [u8]> = BTreeMap::new();
    let store = Arc::new(RwLock::new(Box::new(2)));

    // The & is a borrow so the values can be used more than once
    change_to_value(&store, 3);
    change_to_value(&store, 9);
    change_to_value(&store, 90000);
    change_to_value(&store, 1);

    let mut file = File::open("./sample_data/conversations.json").unwrap();
    let mut file2 = File::create("./sample_data/conversations.bson").unwrap();
    println!("{}", std::fs::metadata("./sample_data/conversations.json").unwrap().len());
    let value: serde_json::Value = serde_json::from_reader(file).unwrap();
    let doc = to_bson(value);

    // let json = serde_json::to_value(Bson::Document(doc)).unwrap();

    let mut db = init_db();
    db.insert(doc);

    // write_test_doc();
    // let mut file = File::open("test.bson");

    // println!("{}", read_document_size(file.unwrap()).unwrap());
    // println!("{}", std::fs::metadata("test.bson").unwrap().len());
    // let json = serde_json::to_value(Bson::Document(doc)).unwrap();
    // println!("{}", json);  // {"date":"2021-02-20T00:36:46.597117805Z"}

    /*
    let doc =  doc! { "test": Bson::String(String::from("test")) };
    //let mut byte_array : Vec<u8> = vec![];
    //doc.to_writer(&mut byte_array);

    let mut super_page = DataPage { page: Page::new(PageType::Super,
        0,
        0,
        0,
        0); }

    // 3m
    for idx in 0..3000000 {
        super_page.add_document( doc.clone());
    }

    super_page.scan_documents2();
    */
    // let config = Configuration::standard();
    /*
    println!("Saving ...");
    let mut file = File::create("test3.db").unwrap();
    bincode::serialize_into(&mut file, &super_page);
    file.flush();
    println!("Done.")
    */
    // bincode::encode_into_std_write(&super_page, &mut file, config);
}

fn write_test_doc() {
    if std::path::Path::new("test.bson").exists() {
        return;
    }

    let doc = doc! { "test": Bson::String(String::from("test")) };
    let mut file = File::create("test.bson");
    let writer = file.unwrap();
    doc.to_writer(&writer);
}

fn read_document_size(mut reader: impl Read + Seek) -> Result<i32, io::Error>  {
    const read_size: usize = 4;

    // Get Buffer
    let mut buf= [0; read_size];

    // Read Size
    reader.read_exact(&mut buf);

    // Rewind
    reader.seek(SeekFrom::Current(read_size as i64))?;

    // BSON Format is Little Endian
    // with the exception of timestamp and counter
    // from_be_bytes -> Big Endian
    // from_le_bytes -> Little Endian
    // from_ne_bytes -> Native Endian
    Ok(i32::from_le_bytes(buf))
}

fn example(mut reader: impl Read + Seek, num_bytes: i64) -> io::Result<String> {
    reader.seek(SeekFrom::Current(num_bytes))?;

    let mut s = String::new();
    reader.take(5).read_to_string(&mut s)?;

    Ok(s)
}

fn change_to_value(store: &Arc<RwLock<Box<i32>>>, new_value: i32) {

    let mut l = store.write().unwrap();
    let boxed = &mut *l;
    let m = boxed.as_mut();
    let original = m.clone();
    *m = new_value;
    println!("Values: {}, {}", original, m);
}