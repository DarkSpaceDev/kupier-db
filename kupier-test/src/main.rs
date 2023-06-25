use kupier_core::storage::rocksdb::Datastore;
use kupier_engine::{
    execution::{self, Executor, QueryPlan},
    plan::CollectionScan,
};
use std::time::Instant;

// https://stackoverflow.com/questions/61699050/how-can-i-make-my-rust-code-run-faster-in-parallel
// https://stackoverflow.com/questions/68547268/cannot-borrow-data-in-an-arc-as-mutable

// use std::sync::atomic::AtomicUsize;
// static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

async fn test_basic_query(collection: &str, ex: &Executor) {
    let now = Instant::now();

    let query_result = ex
        .execute_select(QueryPlan {
            schema: String::from("default"),
            collection: String::from(collection),
        })
        .await
        .expect("Failed to execute query.");

    let microseconds = now.elapsed().as_micros();
    println!(
        "Elapsed: {}μs - {} records.",
        microseconds,
        query_result.records.len()
    );
}

async fn test_load_all(collection: &str, ex: &Executor) {
    let now = Instant::now();

    let query_result = ex
        .execute_collection_scan(CollectionScan {
            schema: String::from("default"),
            collection: String::from(collection),
            // source: Option::None,
            alias: Option::None,
            expr: Option::None,
        })
        .await
        .expect("Failed to execute query.");

    let microseconds = now.elapsed().as_micros();
    println!(
        "Elapsed: {}μs - {} records.",
        microseconds,
        query_result.len()
    );
}

async fn test_count_all(collection: &str, ex: &Executor) {
    let now = Instant::now();

    let query_result = ex
        .execute_count(CollectionScan {
            schema: String::from("default"),
            collection: String::from(collection),
            // source: Option::None,
            alias: Option::None,
            expr: Option::None,
        })
        .await
        .expect("Failed to execute query.");

    let microseconds = now.elapsed().as_micros();
    println!("Elapsed: {}μs - {} records.", microseconds, query_result);
}

async fn test_builk_insert(collection: &str, ex: &Executor, count: u32) {
    let now = Instant::now();

    ex.test_bulk_insert(String::from(collection), count)
        .await
        .unwrap();

    let microseconds = now.elapsed().as_micros();
    let seconds = now.elapsed().as_secs();

    println!("Elapsed: {}μs - {} records.", microseconds, count);
    println!("Elapsed: {}s - {} records.", seconds, count);
}

#[tokio::main]
async fn main() {
    let mut command: String = String::from("");

    let ds = Datastore::new(".\\dbs\\vuln_db").await.unwrap();
    let ex = Executor::new(ds);

    // ex.create_collection(String::from("test_collection"))
    //     .await
    //     .unwrap();

    for _ in 0..100 {
        ex.test_insert(String::from("coal_mine")).await.unwrap();
    }

    for _ in 0..1000 {
        ex.test_insert(String::from("test_collection"))
            .await
            .unwrap();
    }

    for _ in 0..25 {
        ex.test_insert(String::from("test_collection2"))
            .await
            .unwrap();
    }

    test_builk_insert("test_collection3", &ex, 1000).await;
    test_builk_insert("test_collection3", &ex, 10000).await;
    test_builk_insert("test_collection3", &ex, 100000).await;

    loop {
        let mut prompt = "kupier >> ";
        if command.len() > 0 {
            prompt = "";
        }

        let input = user_input::get_input(prompt);

        if input == "exit" {
            break;
        }

        if input != "" {
            if command.len() > 0 {
                command.push_str("\n");
            }

            command.push_str(&input);
            continue;
        }

        let local_command = command.clone();
        command.clear();

        println!("------------------------------------------------------");
        println!("COMMAND:");
        println!("------------------------------------------------------");
        println!("{}", local_command);
        println!("------------------------------------------------------");
        println!();

        let now = Instant::now();
        let result = kupier_lang::parser::parse_query(&local_command);
        let microseconds = now.elapsed().as_micros();

        println!("Elapsed: {}μs", microseconds);

        test_basic_query("test_collection", &ex).await;
        test_basic_query("test_collection2", &ex).await;
        test_load_all("test_collection", &ex).await;
        test_count_all("test_collection", &ex).await;

        if result.is_ok() {
            println!("{:?}", result.unwrap());
        } else {
            println!("{:?}", result.err().unwrap())
        }
    }
}

mod user_input {
    use std::io::{self, Write};
    pub fn get_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().lock().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {}
            Err(_no_updates_is_fine) => {}
        }
        input.trim().to_string()
    }
}
