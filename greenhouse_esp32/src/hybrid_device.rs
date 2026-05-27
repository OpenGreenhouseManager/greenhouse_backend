use std::sync::Arc;

use esp_idf_svc::http::server::{Configuration, EspHttpServer, Method};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::device_builder::DeviceState;
use crate::error::{Error, Result};
use crate::handler;

pub fn start_server<T>(
    state: Arc<DeviceState<T>>,
    config: Configuration,
) -> Result<EspHttpServer<'static>>
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    let mut server = EspHttpServer::new(&config).map_err(Error::Esp)?;

    {
        let s = state.clone();
        server
            .fn_handler("/read", Method::Get, move |req| handler::handle_read(&s, req))
            .map_err(Error::Esp)?;
    }
    {
        let s = state.clone();
        server
            .fn_handler("/write", Method::Post, move |req| handler::handle_write(&s, req))
            .map_err(Error::Esp)?;
    }
    {
        let s = state.clone();
        server
            .fn_handler("/status", Method::Get, move |req| handler::handle_status(&s, req))
            .map_err(Error::Esp)?;
    }
    {
        let s = state.clone();
        server
            .fn_handler("/config", Method::Get, move |req| handler::handle_get_config(&s, req))
            .map_err(Error::Esp)?;
    }
    {
        let s = state.clone();
        let nvs = state.nvs.clone();
        server
            .fn_handler("/config", Method::Post, move |req| {
                handler::handle_post_config(&s, &nvs, req)
            })
            .map_err(Error::Esp)?;
    }
    {
        let s = state.clone();
        let nvs = state.nvs.clone();
        server
            .fn_handler("/activate", Method::Post, move |req| {
                handler::handle_activate(&s, &nvs, req)
            })
            .map_err(Error::Esp)?;
    }

    Ok(server)
}
