mod route;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use route::routes::{clear_cache, fetch_data};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    let cache: HashMap<String, Value> = HashMap::new();
    let sql_repo = Arc::new(Mutex::new(cache));
    let sql_data = Data::new(sql_repo);

    println!("Running on server http://localhost:3000/");

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(sql_data.clone())
            .service(fetch_data)
            .service(clear_cache)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
