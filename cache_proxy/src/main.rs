use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Parser)]
struct Cli {
    #[arg(long)] // this makes the argument a named flag
    port: Option<String>,

    #[arg(long)]
    origin: Option<String>,

    #[arg(long)]
    clear_cache: bool,
}

#[derive(Serialize, Deserialize)]
struct CacheData {
    cache_response: String,
    data: Value,
}

fn main() {
    let args = Cli::parse();
    let client = Client::new();

    if args.clear_cache {
        let res = client.delete("http://localhost:3000/clear-cache").send();

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Cache cleared");
                } else {
                    println!("Failed to clear cache. Status: {}", response.status());
                }
            }
            Err(e) => println!("Failed to handle request: {}", e),
        }
    } else {
        match args.origin {
            Some(origin) => {
                // Skip async/await because this
                let res = client
                    .put(format!("http://localhost:{}/", args.port.clone().unwrap()))
                    .header("Content-Type", "text/plain")
                    .body(origin.clone())
                    .send();
                match res {
                    Ok(response) => {
                        let result = response
                            .headers()
                            .get("x-cache")
                            .map(|val| val.to_str().unwrap_or("").to_string());

                        let stringified_json = response.text().unwrap_or(String::from(""));
                        let json_object: Value = serde_json::from_str(&stringified_json)
                            .unwrap_or(Value::Object(Default::default()));

                        if let Some(cache_header) = result {
                            // println!("X-Cache: {}\n\n{:#?}", cache_header, json_object);
                            println!("X-Cache: {}\n\n{}", cache_header, json_object);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            None => {
                eprintln!("Error: --origin is required when --clear-cache is false.");
                std::process::exit(1);
            }
        }
    }
}
