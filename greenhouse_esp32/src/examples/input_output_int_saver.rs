//! ESP32 port of the `input_output_int_saver` example.
//!
//! Porting diff from the Linux/Axum version:
//!   - Remove `#[tokio::main]` and `async fn main` → plain `fn main`
//!   - Add ESP-IDF boilerplate (link_patches, logger, WiFi, NVS partition)
//!   - Remove `async` from all four handler functions
//!   - Replace `axum::http::StatusCode` with `greenhouse_esp32::StatusCode`
//!   - Replace `init_hybrid_router + axum::serve` with `DeviceBuilder::run()`
//!   - Pass a `default_config` to the builder instead of a config file path
//!   - Drop `.await` from `trigger_alert` calls (sync on ESP32)
//!
//! For PWM / motor control see the comment block at the bottom of the file.

use std::sync::Arc;

use greenhouse_core::smart_device_dto::{
    Type,
    config::{ConfigRequestDto, TypeOption},
    status::{DeviceStatusDto, DeviceStatusResponseDto},
};
use greenhouse_esp32::{Config, DeviceBuilder, StatusCode};
use serde::{Deserialize, Serialize};

static SAVED_NUMBER: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(20);

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

fn main() {
    // ── ESP-IDF boilerplate ────────────────────────────────────────────────
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take().unwrap();
    let nvs_partition = esp_idf_svc::nvs::EspDefaultNvsPartition::take().unwrap();

    // ── WiFi (fill in your SSID/password or load from NVS) ────────────────
    let _wifi = connect_wifi(peripherals.modem, sysloop, nvs_partition.clone());

    // ── Device ────────────────────────────────────────────────────────────
    let default_config = Config {
        port: 6001,
        datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".to_string(),
        additional_config: ExampleDeviceConfig { min: 0, max: 100 },
        scripting_api: None,
    };

    let device = DeviceBuilder::new_hybrid_device(
        nvs_partition,
        default_config,
        read_handler,
        write_handler,
        status_handler,
        config_interceptor_handler,
        TypeOption::Number,
        TypeOption::Number,
    )
    .unwrap();

    // Blocks forever, serving HTTP on the configured port.
    device.run().unwrap();
}

fn read_handler(_config: Arc<Config<ExampleDeviceConfig>>) -> Type {
    Type::Number(SAVED_NUMBER.load(std::sync::atomic::Ordering::Relaxed) as f64)
}

fn write_handler(data: Type, config: Arc<Config<ExampleDeviceConfig>>) -> StatusCode {
    let number = match data {
        Type::Number(n) => n,
        _ => return StatusCode::BAD_REQUEST,
    };
    let number_i32 = number as i32;
    if config.additional_config.min > number_i32 || config.additional_config.max < number_i32 {
        return StatusCode::BAD_REQUEST;
    }
    SAVED_NUMBER.store(number_i32, std::sync::atomic::Ordering::Relaxed);
    StatusCode::OK
}

fn status_handler(config: Arc<Config<ExampleDeviceConfig>>) -> DeviceStatusResponseDto {
    DeviceStatusResponseDto {
        status: DeviceStatusDto::Online,
        datasource_id: config.datasource_id.clone(),
    }
}

fn config_interceptor_handler(
    req: ConfigRequestDto<ExampleDeviceConfig>,
    old: Arc<Config<ExampleDeviceConfig>>,
) -> Config<ExampleDeviceConfig> {
    Config {
        additional_config: req.additional_config,
        ..(*old).clone()
    }
}

fn connect_wifi(
    modem: esp_idf_hal::modem::Modem,
    sysloop: esp_idf_svc::eventloop::EspSystemEventLoop,
    nvs: esp_idf_svc::nvs::EspDefaultNvsPartition,
) -> esp_idf_svc::wifi::EspWifi<'static> {
    use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs)).unwrap(),
        sysloop,
    )
    .unwrap();

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "YourSSID".try_into().unwrap(),
        password: "YourPassword".try_into().unwrap(),
        ..Default::default()
    }))
    .unwrap();

    wifi.start().unwrap();
    wifi.connect().unwrap();
    wifi.wait_netif_up().unwrap();

    log::info!(
        "WiFi connected, IP: {:?}",
        wifi.wifi().sta_netif().get_ip_info().unwrap()
    );

    wifi.into_inner()
}

// ── PWM / motor control example ───────────────────────────────────────────
//
// For a PWM-controlled motor, swap the `AtomicI32` for an `Arc<Mutex<LedcDriver>>`
// and capture it in the write handler:
//
//   use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig};
//   use std::sync::{Arc, Mutex};
//
//   let timer = LedcTimerDriver::new(peripherals.ledc.timer0, &TimerConfig::default()).unwrap();
//   let motor = Arc::new(Mutex::new(
//       LedcDriver::new(peripherals.ledc.channel0, &timer, peripherals.pins.gpio5).unwrap(),
//   ));
//
//   let motor_write = motor.clone();
//   let write_handler = move |data: Type, config: Arc<Config<MotorConfig>>| {
//       if let Type::Number(speed_pct) = data {
//           let mut drv = motor_write.lock().unwrap();
//           let max = drv.get_max_duty();
//           let duty = (speed_pct.clamp(0.0, 100.0) / 100.0 * max as f64) as u32;
//           drv.set_duty(duty).ok();
//           StatusCode::OK
//       } else {
//           StatusCode::BAD_REQUEST
//       }
//   };
