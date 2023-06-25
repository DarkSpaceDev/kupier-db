use std::time::Instant;

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use kupier_core::storage::rocksdb::Datastore;
use kupier_engine::{
    execution::{Executor, QueryResult},
    plan::QueryPlan,
};
use kupier_lang::ast::Node;
use serde::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Command {
    operation: String,
    command: String,
}

#[derive(Serialize, Deserialize)]
struct CommandResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Vec<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    execution_plan: Option<Vec<Node>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_elapsed_µs: Option<u128>,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_elapsed_ms: Option<u128>,
}

#[post("/")]
async fn execute(ds: web::Data<Datastore>, command: web::Json<Command>) -> impl Responder {
    let dsx = ds.into_inner();
    let dsr = dsx.as_ref();
    let ex = Executor::new(dsr.clone());

    let now = Instant::now();
    let result = kupier_lang::parser::parse_query(&command.command);

    let mut command_result = CommandResult {
        result: Option::None,
        error: Option::None,
        execution_plan: Option::None,
        time_elapsed_ms: Some(now.elapsed().as_millis()),
        time_elapsed_µs: Some(now.elapsed().as_micros()),
    };

    if (result.is_ok()) {
        command_result.execution_plan = result.clone().ok();

        match result.clone().ok().unwrap().first().unwrap() {
            Node::Query(q) => {
                // Execute Query
                let query_plan = QueryPlan::from_ast(q);
                match query_plan.0 {
                    kupier_engine::plan::Node::CollectionScan(x) => {
                        let execution_plan = kupier_engine::execution::QueryPlan {
                            collection: x.collection.clone(),
                            schema: x.schema.clone(),
                        };

                        let query_result: Result<
                            kupier_engine::execution::QueryResult,
                            kupier_core::error::Error,
                        > = ex.execute_select(execution_plan).await;

                        if (query_result.is_err()) {
                            command_result.error =
                                Some(format!("Execution Error: {:?}", query_result.err()))
                        } else {
                            let mut records: Vec<Value> = Vec::new();

                            for bson in query_result.ok().unwrap().records {
                                records.push(bson.into_canonical_extjson());
                            }

                            command_result.result = Some(records);
                        }
                    }
                    invalid => command_result.error = Some(format!("Invalid Plan! {:?}", invalid)),
                }
            }

            invalid => command_result.error = Some(format!("Invalid Rule! {:?}", invalid)),
        }
    } else {
        command_result.error = Some(format!("{:?}", result.err()));
    }

    HttpResponse::Ok().json(command_result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ds = web::Data::new(Datastore::new(".\\dbs\\vuln_db").await.unwrap());
    HttpServer::new(move || App::new().app_data(ds.clone()).service(execute))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
