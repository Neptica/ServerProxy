use actix_web::{
    delete, http::header, http::header::ContentType, put, web::Data, Error, HttpResponse,
    ResponseError,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct CacheData {
    cache_response: String,
    data: Value,
}

#[derive(Debug, Display)]
struct ProxyError {
    error: String,
}

impl ResponseError for ProxyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

// HttpResponse::Ok()
//     .insert_header((header::HeaderName::from_static("x-cache"), cache_info))
//     .json(data)

async fn fetch(
    origin: String,
    cache_pool: Data<Arc<Mutex<HashMap<String, Value>>>>,
) -> Result<HttpResponse, ProxyError> {
    let mut cache = cache_pool.lock().await;
    if let Some(cached_response) = cache.get(&origin) {
        Ok(HttpResponse::Ok()
            .insert_header((header::HeaderName::from_static("x-cache"), "HIT"))
            .json(cached_response.clone()))
    } else {
        println!("URL: {}", origin);
        match reqwest::get(&origin).await {
            Ok(grabbed_response) => {
                if grabbed_response.status().is_success() {
                    match grabbed_response.json::<Value>().await {
                        Ok(parsed_body) => {
                            if parsed_body.is_null() {
                                return Err(ProxyError {
                                    error: String::from("Invalid URL"),
                                });
                            } else {
                                cache.insert(origin, parsed_body.clone());
                                return Ok(HttpResponse::Ok()
                                    .insert_header((
                                        header::HeaderName::from_static("x-cache"),
                                        "MISS",
                                    ))
                                    .json(parsed_body.clone()));
                            }
                        }
                        Err(_) => {
                            return Err(ProxyError {
                                error: String::from("Internal Server Error"),
                            });
                        }
                    }
                }
                Err(ProxyError {
                    error: format!("Request failed: {}", grabbed_response.status()),
                })
            }
            Err(e) => {
                println!("{}", e);
                Err(ProxyError {
                    error: String::from("Internal Server Error: Unable to fetch data"),
                })
            }
        }
    }
}

#[put("/")]
pub async fn fetch_data(
    cache_pool: Data<Arc<Mutex<HashMap<String, Value>>>>,
    un_req: String,
) -> Result<HttpResponse, ProxyError> {
    println!("Received request");
    let req = un_req.strip_suffix('/').unwrap_or(&un_req);

    fetch(String::from(req), cache_pool).await
}

#[delete("/clear-cache")]
pub async fn clear_cache(
    cache_pool: Data<Arc<Mutex<HashMap<String, Value>>>>,
) -> Result<String, Error> {
    let mut cache = cache_pool.lock().await;
    cache.clear();
    Ok("Cleared Cache".to_string())
}
