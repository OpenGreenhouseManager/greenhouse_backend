#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
package.edition = "2024"
[dependencies]
time = "0.1.25"
chrono = { version = "0.4", features = ["serde"] }
greenhouse_core = "0.0.7"
rand = "0.9.1"
reqwest = {version = "0.12.15", features = ["json"]}
tokio = { version = "1.44.2", features = ["macros", "rt-multi-thread"] }
futures = "0.3"
---

use chrono::{DateTime, Days, Local, NaiveDate, NaiveDateTime, Utc};
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_entry::PostDiaryEntryDtoRequest;
use rand::Rng;
use std::env;
use std::time::Duration;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    if (env::args().len() < 1) {
        println!("Usage: generate_data <url>");
        return;
    }
    let url = env::args().nth(1).unwrap();
    println!("Generating data for url: {}", url);
    generate_diary_entries(url).await;
}

async fn generate_diary_entries(url: String) {
    let mut rng = rand::thread_rng();
    let mut requests = Vec::new();

    for i in 0..50 {
        let entry = PostDiaryEntryDtoRequest {
            title: format!("Diary Entry number {}", i),
            content: format!("Diary Entry {}", rng.gen_range(1..100)),

            date: (Utc::now() - Days::new(rng.gen_range(0..30))).to_string(),
        };
        
        requests.push(entry);
    }
    let mut futures = Vec::new();

    for entry in requests {;
        let future = reqwest::Client::new()
                .post(url.to_string() + "/diary")
                .json(&entry)
                .send();
        futures.push(future);
    }
    
    let res = join_all(futures).await;

    for response in res {
        match response {
            Ok(res) => {
                println!("Response: {:?}", res);
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
            
}
