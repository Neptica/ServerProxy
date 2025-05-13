use actix_web::{
    delete, http::header::ContentType, put, web::Data, web::Json, Error, HttpResponse,
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

async fn fetch(
    origin: String,
    cache_pool: Data<Arc<Mutex<HashMap<String, Value>>>>,
) -> Result<CacheData, ProxyError> {
    let mut cache = cache_pool.lock().await;
    if let Some(cached_response) = cache.get(&origin) {
        let response: CacheData = CacheData {
            cache_response: String::from("X-cache: HIT"),
            data: cached_response.clone(),
        };
        Ok(response)
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
                                let response: CacheData = CacheData {
                                    cache_response: String::from("X-cache: MISS"),
                                    data: parsed_body,
                                };
                                return Ok(response);
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
) -> Result<Json<CacheData>, ProxyError> {
    println!("Received request");
    let req = un_req.strip_suffix('/').unwrap_or(&un_req);

    match fetch(String::from(req), cache_pool).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err(e),
    }
}

#[delete("/clear-cache")]
pub async fn clear_cache(
    cache_pool: Data<Arc<Mutex<HashMap<String, Value>>>>,
) -> Result<String, Error> {
    let mut cache = cache_pool.lock().await;
    cache.clear();
    Ok("Cleared Cache".to_string())
}
