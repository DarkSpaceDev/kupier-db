use std::sync::atomic::AtomicUsize;
use std::time::Instant;
use uuid::Uuid;

// https://stackoverflow.com/questions/61699050/how-can-i-make-my-rust-code-run-faster-in-parallel
// https://stackoverflow.com/questions/68547268/cannot-borrow-data-in-an-arc-as-mutable

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() {
    println!("Running Test...");

    let mut datastore =
        kupier_core::storage::rocksdb::Datastore::new("D:\\workspace\\kuiper2-rs\\dbs\\vuln_db")
            .await
            .unwrap();

    datastore.drop_index("chimmi_churri");

    let now = Instant::now();

    for i in 1..10000 {
        let mut txn = datastore.transaction(true).await.unwrap();
        txn.insert(*Uuid::new_v4().as_bytes(), *Uuid::new_v4().as_bytes())
            .await
            .unwrap();
        txn.commit().await.unwrap();
    }

    let elapsed = now.elapsed();
    println!("[{}]", elapsed.as_micros() / 10000);
}
