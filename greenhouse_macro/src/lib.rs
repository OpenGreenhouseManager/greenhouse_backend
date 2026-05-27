//! Procedural macros for the OpenGreenhouseManager backend.
//!
//! # Macros
//!
//! | Macro | Kind | Purpose |
//! |-------|------|---------|
//! | [`authenticate`] | Attribute | Wrap an Axum route handler with JWT-based role enforcement |
//! | [`IntoJsonResponse`] | Derive | Implement `axum::response::IntoResponse` via `axum::Json` |
//!
//! # Quick start
//!
//! ```rust,ignore
//! use greenhouse_macro::{authenticate, IntoJsonResponse};
//! use serde::{Deserialize, Serialize};
//!
//! // Derive JSON serialisation + IntoResponse in one step.
//! #[derive(Serialize, Deserialize, IntoJsonResponse)]
//! pub struct MyResponse {
//!     pub message: String,
//! }
//!
//! // Restrict a route to users with the ADMIN role.
//! #[authenticate("ADMIN")]
//! pub async fn admin_only_handler(
//!     cookies: tower_cookies::Cookies,
//! ) -> Result<MyResponse, crate::helper::error::Error> {
//!     Ok(MyResponse { message: "hello admin".to_string() })
//! }
//! ```
//!
//! The `authenticate` attribute expects `tower_cookies::Cookies` to be an extractor
//! parameter and `crate::helper::error::Error` to expose the required error variants.
//! These are conventions from the OpenGreenhouseManager service crates.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, NestedMeta};

/// Wraps an Axum route handler to enforce JWT-based role authorisation.
///
/// The attribute reads the `auth-token` HTTP-only cookie, decodes the JWT, and
/// checks that `claims.role == expected_role`. If the check fails the wrapped
/// function returns early with `Error::AdminRoute` (or `Error::CookieNotFound`
/// when the cookie is absent).
///
/// # Arguments
///
/// Takes a single string literal — the required role name:
///
/// ```rust,ignore
/// #[authenticate("ADMIN")]
/// pub async fn my_handler(
///     cookies: tower_cookies::Cookies,
/// ) -> Result<MyResponse, Error> { ... }
/// ```
///
/// # Requirements in the calling crate
///
/// - `crate::helper::error::Error` with variants `CookieNotFound` and `AdminRoute`
/// - `crate::auth::AUTH_TOKEN` — the cookie name constant
/// - `crate::helper::token::get_claims(token)` — JWT decoder returning a struct
///   with a `role: String` field
#[proc_macro_attribute]
pub fn authenticate(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    // Extract role string from attribute like #[authenticate("ADMIN")]
    let expected_role = match args.first() {
        Some(NestedMeta::Lit(Lit::Str(s))) => s.value(),
        _ => {
            return syn::Error::new_spanned(
                args.first().unwrap(),
                "expected role as a string, e.g. #[authenticate(\"ADMIN\")]",
            )
            .to_compile_error()
            .into();
        }
    };

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;

    let output = quote! {
        #(#attrs)*
        #vis #sig {
            use axum::http::StatusCode;
            use tower_cookies::Cookies;
            use crate::helper::error::{Error};
            use crate::{ auth::AUTH_TOKEN};
            use crate::helper;

            if let Ok(token) = cookies
                .get(AUTH_TOKEN)
                .map(|c| c.value().to_string())
                .ok_or(Error::CookieNotFound)
            {
                let claims = helper::token::get_claims(token)?;
                if claims.role != #expected_role  {
                    return Err(Error::AdminRoute.into());
                }
            }

            #block
        }
    };

    output.into()
}

/// Derives `axum::response::IntoResponse` by serialising `self` as JSON.
///
/// The generated implementation wraps the value in [`axum::Json`] and delegates
/// to its `IntoResponse`. This is equivalent to writing:
///
/// ```rust,ignore
/// impl axum::response::IntoResponse for MyStruct {
///     fn into_response(self) -> axum::response::Response {
///         axum::Json(self).into_response()
///     }
/// }
/// ```
///
/// # Requirements
///
/// The type must implement `serde::Serialize`.
///
/// # Example
///
/// ```rust,ignore
/// use greenhouse_macro::IntoJsonResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize, IntoJsonResponse)]
/// pub struct DeviceListResponse {
///     pub devices: Vec<String>,
/// }
/// ```
#[proc_macro_derive(IntoJsonResponse)]
pub fn into_response_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_into_response_macro(&ast)
}

fn impl_into_response_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generated = quote! {

        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                axum::Json(self).into_response()
            }
        }
    };
    generated.into()
}
