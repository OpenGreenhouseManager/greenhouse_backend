use std::sync::{Arc, Mutex};

use embedded_io::{Read, Write};
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::io::EspIOError;
use greenhouse_core::smart_device_dto::{
    Type,
    config::{ConfigRequestDto, ConfigResponseDto, TypeOption},
    read::ReadResponseDto,
    status::DeviceStatusResponseDto,
    write::WriteRequestDto,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::config::{Config, ScriptingApi, write_nvs_config};
use crate::device_builder::DeviceState;
use crate::StatusCode;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};

const BODY_BUF_SIZE: usize = 4096;

fn read_body(req: &mut Request<&mut EspHttpConnection>) -> Vec<u8> {
    let mut buf = vec![0u8; BODY_BUF_SIZE];
    let mut total = 0usize;
    loop {
        match req.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if total >= buf.len() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    buf.truncate(total);
    buf
}

fn write_json_response(
    req: Request<&mut EspHttpConnection>,
    status: u16,
    body: &[u8],
) -> Result<(), EspIOError> {
    req.into_response(status, None, &[("Content-Type", "application/json")])?
        .write_all(body)
}

pub fn handle_read<T>(
    state: &Arc<DeviceState<T>>,
    req: Request<&mut EspHttpConnection>,
) -> Result<(), EspIOError>
where
    T: Clone + Default + Serialize + Send + Sync + 'static,
{
    let read_handler = state
        .read_handler
        .as_ref()
        .expect("read handler missing for output device");
    let config = Arc::new(state.config.lock().unwrap().clone());
    let data = read_handler(config);
    let body =
        serde_json::to_vec(&ReadResponseDto { data }).expect("ReadResponseDto serialization");
    write_json_response(req, 200, &body)
}

pub fn handle_write<T>(
    state: &Arc<DeviceState<T>>,
    mut req: Request<&mut EspHttpConnection>,
) -> Result<(), EspIOError>
where
    T: Clone + Default + Serialize + Send + Sync + 'static,
{
    let write_handler = state
        .write_handler
        .as_ref()
        .expect("write handler missing for input device");
    let body = read_body(&mut req);
    let payload: WriteRequestDto = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(_) => return write_json_response(req, 400, b"{}"),
    };
    let config = Arc::new(state.config.lock().unwrap().clone());
    let status = write_handler(payload.data, config);
    write_json_response(req, status.as_u16(), b"{}")
}

pub fn handle_status<T>(
    state: &Arc<DeviceState<T>>,
    req: Request<&mut EspHttpConnection>,
) -> Result<(), EspIOError>
where
    T: Clone + Default + Serialize + Send + Sync + 'static,
{
    let config = Arc::new(state.config.lock().unwrap().clone());
    let dto: DeviceStatusResponseDto = (state.status_handler)(config);
    let body = serde_json::to_vec(&dto).expect("DeviceStatusResponseDto serialization");
    write_json_response(req, 200, &body)
}

pub fn handle_get_config<T>(
    state: &Arc<DeviceState<T>>,
    req: Request<&mut EspHttpConnection>,
) -> Result<(), EspIOError>
where
    T: Clone + Default + Serialize + Send + Sync + 'static,
{
    let config = state.config.lock().unwrap();
    let (input_type, output_type, mode) = match &state.mode {
        crate::device_builder::Mode::Input(t) => (
            Some(t.clone()),
            None,
            greenhouse_core::smart_device_dto::config::Mode::Input,
        ),
        crate::device_builder::Mode::Output(t) => (
            None,
            Some(t.clone()),
            greenhouse_core::smart_device_dto::config::Mode::Output,
        ),
        crate::device_builder::Mode::InputOutput(i, o) => (
            Some(i.clone()),
            Some(o.clone()),
            greenhouse_core::smart_device_dto::config::Mode::InputOutput,
        ),
        crate::device_builder::Mode::Unknown => (
            None,
            None,
            greenhouse_core::smart_device_dto::config::Mode::Unknown,
        ),
    };
    let dto = ConfigResponseDto {
        mode,
        input_type,
        output_type,
        scripting_api: config.scripting_api.as_ref().map(|s| {
            greenhouse_core::smart_device_dto::config::ScriptingApi {
                url: s.url.clone(),
                token: s.token.clone(),
            }
        }),
        additional_config: config.additional_config.clone(),
    };
    let body = serde_json::to_vec(&dto).expect("ConfigResponseDto serialization");
    drop(config);
    write_json_response(req, 200, &body)
}

pub fn handle_post_config<T>(
    state: &Arc<DeviceState<T>>,
    nvs: &Arc<Mutex<EspNvs<NvsDefault>>>,
    mut req: Request<&mut EspHttpConnection>,
) -> Result<(), EspIOError>
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    let body = read_body(&mut req);
    let req_dto: ConfigRequestDto<T> = match serde_json::from_slice(&body) {
        Ok(d) => d,
        Err(_) => return write_json_response(req, 400, b"{}"),
    };
    let old_config = Arc::new(state.config.lock().unwrap().clone());
    let new_config = (state.config_interceptor)(req_dto, old_config);
    {
        let mut guard = state.config.lock().unwrap();
        *guard = new_config.clone();
    }
    if let Ok(mut nvs_guard) = nvs.lock() {
        let _ = write_nvs_config(&mut nvs_guard, &new_config);
    }
    write_json_response(req, 200, b"{}")
}

pub fn handle_activate<T>(
    state: &Arc<DeviceState<T>>,
    nvs: &Arc<Mutex<EspNvs<NvsDefault>>>,
    mut req: Request<&mut EspHttpConnection>,
) -> Result<(), EspIOError>
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    use greenhouse_core::smart_device_dto::activation::ActivateRequestDto;
    let body = read_body(&mut req);
    let dto: ActivateRequestDto = match serde_json::from_slice(&body) {
        Ok(d) => d,
        Err(_) => return write_json_response(req, 400, b"{}"),
    };
    {
        let mut guard = state.config.lock().unwrap();
        guard.scripting_api = Some(ScriptingApi {
            url: dto.url,
            token: dto.token,
        });
        let updated = guard.clone();
        if let Ok(mut nvs_guard) = nvs.lock() {
            let _ = write_nvs_config(&mut nvs_guard, &updated);
        }
    }
    write_json_response(req, 200, b"{}")
}
