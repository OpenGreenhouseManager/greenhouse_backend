//! Demonstrates the [`HttpErrorMapping`] trait and [`HttpErrorResponse`] type.
//!
//! Shows how to define a service error enum and integrate it with the
//! standardised HTTP error response system.
//!
//! Run with:
//! ```bash
//! cargo run --example http_error_mapping --features error_handling
//! ```

use axum::http::StatusCode;
use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
use greenhouse_core::impl_http_error_from;
use std::fmt;
use std::num::ParseIntError;

/// A simple service error enum.
#[derive(Debug)]
enum MyError {
    NotFound,
    BadRequest(String),
    ParseError(ParseIntError),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::NotFound => write!(f, "resource not found"),
            MyError::BadRequest(msg) => write!(f, "bad request: {msg}"),
            MyError::ParseError(e) => write!(f, "parse error: {e}"),
        }
    }
}

impl HttpErrorMapping for MyError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            MyError::NotFound => StatusCode::NOT_FOUND,
            MyError::BadRequest(_) => StatusCode::BAD_REQUEST,
            MyError::ParseError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<ParseIntError> for MyError {
    fn from(e: ParseIntError) -> Self {
        MyError::ParseError(e)
    }
}

// Use the macro to implement From<ParseIntError> for HttpErrorResponse<MyError>.
impl_http_error_from!(MyError { ParseIntError });

fn main() {
    // Construct via From trait
    let resp: HttpErrorResponse<MyError> = MyError::NotFound.into();
    println!("Status code: {}", resp.error.to_status_code());
    println!("Message:     {}", resp.error.to_error_message());

    // Construct explicitly
    let resp2 = HttpErrorResponse::new(MyError::BadRequest("missing field".to_string()));
    println!("Status code: {}", resp2.error.to_status_code());
    println!("Message:     {}", resp2.error.to_error_message());

    // Construct from a source error type via impl_http_error_from!
    let parse_err: Result<i32, _> = "not_a_number".parse::<i32>();
    let resp3: HttpErrorResponse<MyError> = parse_err.unwrap_err().into();
    println!("Status code: {}", resp3.error.to_status_code());
    println!("Message:     {}", resp3.error.to_error_message());
}
