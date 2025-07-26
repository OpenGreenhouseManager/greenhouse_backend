#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
package.edition = "2024"
[dependencies]
time = "0.1.25"
chrono = { version = "0.4", features = ["serde"] }
greenhouse_core = "0.0.9"
rand = "0.9.1"
reqwest = {version = "0.12.15", features = ["json"]}
tokio = { version = "1.44.2", features = ["macros", "rt-multi-thread"] }
futures = "0.3" 
uuid = { version ="1.16.0", features = [
    "v4",                
    "fast-rng",          
    "macro-diagnostics", 
    "serde"
] }
---
// cargo +nightly -Zscript scripts/generate_data.rs http://localhost:5001 < --diary > < --alert >

use chrono::{DateTime, Days, Local, NaiveDate, NaiveDateTime, Utc};
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_entry::PostDiaryEntryDtoRequest;
use greenhouse_core::data_storage_service_dto::alert_dto::{
    post_create_alert::CreateAlertDto,
};
use rand::Rng;
use std::env;
use std::time::Duration;
use futures::future::join_all;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    if env::args().len() < 1 {
        println!("Usage: generate_data <url>");
        return;
    }
    let url = env::args().nth(1).unwrap();
    println!("Generating data for url: {}", url);

    let args: Vec<String> = env::args().collect();
    if args.contains(&"--diary".into()){
        generate_diary_entries(url.clone()).await;
        println!("Diary entries generated successfully.");
    }
    if args.contains(&"--alerts".into()) {
        generate_alerts(url.clone()).await;
        println!("Alert entries generated successfully.");
    }
    if args.contains(&"--devices".into()) {
        generate_devices(url.clone()).await;
        println!("Device entries generated successfully.");
    }

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

async fn generate_alerts(url: String) {
    let mut rng = rand::thread_rng();
    let mut requests = Vec::new();
    let alert_datasources: Vec<(Uuid, Vec<String>)> = vec![
        (Uuid::parse_str("ecf9ce4f-d58c-4a72-ba51-08da69e3af7d").unwrap(), vec![String::from("Temp"), String::from("Humidity")]),
        (Uuid::parse_str("2b67d459-7b44-4e23-b9cc-12baf43c4440").unwrap(), vec![String::from("Light"), String::from("Soil Moisture")]),
        (Uuid::parse_str("c72db714-d2ca-4c84-b360-202a69832cb2").unwrap(), vec![String::from("CO2"), String::from("Water Level")]),
    ];

    for i in 0..50 {
        let (datasource_ids, identifiers) = &alert_datasources[rng.gen_range(0..alert_datasources.len())];
        let alert = CreateAlertDto {
            identifier: identifiers[rng.gen_range(0..identifiers.len())].to_string(),
            value: Some(rng.gen_range(1..100).to_string()),
            note: Some(format!("Alert note {}", i)),
            datasource_id: datasource_ids.to_string(),
            severity: match rng.gen_range(0..=3) {
                0 => greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Info,
                1 => greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Warning,
                2 => greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Fatal,
                _ => greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity::Error,
            },
        };

        requests.push(alert);
    }
    let mut futures = Vec::new();

    for alert in requests {
        let future = reqwest::Client::new()
            .post(url.to_string() + "/alert")
            .json(&alert)
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

async fn generate_devices(url: String) {
    let mut rng = rand::thread_rng();
    let mut requests = Vec::new();

    for i in 0..50 {
        let device = PostDeviceDtoRequest {
            name: format!("Device number {}", i),
            description: format!("Device description {}", i),
        };

        requests.push(device);
    }
    let mut futures = Vec::new();

    for device in requests {
        let future = reqwest::Client::new()
            .post(url.to_string() + "/device")
            .json(&device)
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