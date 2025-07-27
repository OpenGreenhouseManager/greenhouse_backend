extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, NestedMeta};


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