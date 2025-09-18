mod error;

use error::{Error, Result};
use greenhouse_core::smart_device_dto::{Type, read::ReadResponseDto};
use metrics::gauge;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{AppState, database::device::Device};

pub(crate) fn start_scrape_devices(state: AppState) {
    tokio::spawn(async move {
        let gauge = gauge!(
            "scrape_service_duration",
            &[("device", "scripting_service")]
        );
        loop {
            let scrape_interval = Duration::from_secs(5);
            let now = Instant::now();
            if let Err(e) = scrape_devices(state.clone()).await {
                sentry::capture_error(&e);
                tracing::error!("Error scraping devices: {:?}", e);
            }
            let elapsed = now.elapsed();
            gauge.set(elapsed.as_secs_f64());

            tracing::debug!("Scrape service took {} seconds", elapsed.as_secs_f64());

            if elapsed < scrape_interval {
                tokio::time::sleep(scrape_interval - elapsed).await;
                continue;
            }
            // sentry Error service is to slow
            sentry::capture_error(&Error::ServiceIsTooSlow);
            tracing::error!(
                "Service is too slow and took {} seconds",
                elapsed.as_secs_f64()
            );
        }
    });
}

async fn scrape_devices(state: AppState) -> Result<()> {
    let devices = Device::get_scraping_devices(&state.pool).await?;
    let mut handles = Vec::new();
    for scrape_devices in devices {
        tracing::debug!("Scraping device: {}", scrape_devices.address);
        handles.push(read_device(
            scrape_devices.id,
            scrape_devices.address.clone(),
        ));
    }

    for handle in handles {
        match handle.await {
            Ok(()) => {
                tracing::debug!("Device scraped successfully");
            }
            Err(e) => {
                tracing::error!("Error scraping device: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn read_device(id: Uuid, address: String) -> Result<()> {
    let client = reqwest::Client::new();

    let response: ReadResponseDto = client
        .get(format!("{address}/read"))
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Error scraping device: {:?}", e);

            Error::Request
        })?
        .json()
        .await
        .map_err(|_| Error::Json)?;

    generate_metric(format!("scrape_service_duration_{id}"), response.data);

    Ok(())
}

fn generate_metric(name: String, data: Type) {
    match data {
        Type::Array(_) => {
            tracing::error!("Not implemented: received array");
        }
        Type::Number(data) => {
            let gauge = gauge!(name, &[("type", "number")]);
            gauge.set(data);
        }
        Type::String(data) => {
            let labels = [("type", String::from("string")), ("string_value", data)];
            let gauge = gauge!(name, &labels);
            gauge.set(1);
        }
        Type::Boolean(data) => {
            let gauge = gauge!(name, &[("type", "boolean")]);
            let b = if data { 1.0 } else { 0.0 };
            gauge.set(b);
        }
        Type::Object(data) => {
            for (key, value) in data {
                let next_name = format!("{name}_{key}");
                generate_metric(next_name, value);
            }
        }
        Type::Stream => {
            tracing::debug!("Not implemented: received stream");
        }
        Type::Unknown(_) => {
            tracing::debug!("received unknown");
        }
    }
}
