use actix_web::{delete, get, web::Data, web::Path, Error, Responder};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

async fn fetch(
    path: String,
    cache_pool: Data<Arc<Mutex<HashMap<String, String>>>>,
) -> Result<String, Error> {
    let mut cache = cache_pool.lock().await;
    if let Some(cached_response) = cache.get(&path) {
        let response = format!("{}\n{}", "X-Cache: HIT", cached_response);
        Ok(response)
    } else {
        let url = format!("{}{}", "https://pokeapi.co/api/v2/", path);
        println!("URL: {}", url);
        match reqwest::get(url).await {
            Ok(grabbed_response) => {
                if grabbed_response.status().is_success() {
                    match grabbed_response.text().await {
                        Ok(parsed_body) => {
                            if parsed_body.is_empty() {
                                return Ok("Not a valid URL".to_string());
                            } else {
                                cache.insert(path, parsed_body.clone());
                                let response = format!("{}\n{}", "X-Cache: MISS", parsed_body);
                                return Ok(response);
                            }
                        }
                        Err(_) => {
                            return Ok("Internal Server Error".to_string());
                        }
                    }
                }
                Ok(format!("Request failed: {}", grabbed_response.status()))
            }
            Err(e) => {
                println!("{}", e);
                Ok("Internal Server Error: Unable to fetch data".to_string())
            }
        }
    }
}

#[get("/{tail:.*}")]
pub async fn test(
    cache_pool: Data<Arc<Mutex<HashMap<String, String>>>>,
    tail: Path<String>,
) -> Result<impl Responder, Error> {
    // Normalize the URLs to deal with trailing slashes
    let inner = tail.into_inner();
    let un_path: &str = inner.as_str();
    let path = un_path.strip_suffix('/').unwrap_or(un_path);

    fetch(String::from(path), cache_pool).await
    // Ok(format!("{:?}", cache))
}

#[delete("/clear-cache")]
pub async fn clear_cache(
    cache_pool: Data<Arc<Mutex<HashMap<String, String>>>>,
) -> Result<String, Error> {
    let mut cache = cache_pool.lock().await;
    cache.clear();
    Ok("Cleared Cache".to_string())
}
