use clap::Parser;
use reqwest::{blocking::Client, Error};
use std::{net::TcpStream, process::Command, process::Stdio};

#[derive(Parser)]
struct Cli {
    #[arg(long)] // this makes the argument a named flag
    port: Option<String>,

    #[arg(long)]
    origin: Option<String>,

    #[arg(long)]
    clear_cache: bool,
}

fn fetch(url: String) -> Result<String, Error> {
    reqwest::blocking::get(url)?.text()
}

fn main() {
    let args = Cli::parse();

    if args.clear_cache {
        let res = Client::new()
            .delete("http://localhost:3000/clear-cache")
            .send();

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
            Some(origin) => match fetch(origin) {
                Ok(req) => println!("{}", req),
                Err(e) => println!("{}", e),
            },
            None => {
                eprintln!("Error: --origin is required when --clear-cache is false.");
                std::process::exit(1);
            }
        }
    }
}
