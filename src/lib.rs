//! Library for controlling Yeelight lamps through Rust.
use palette as _;

/// Modules for commands.
pub mod cmd;
/// Module for code related to interfacing with lamps.
pub mod lamp;

mod derive_alias {
    derive_aliases::define! {
        Serde = ::serde::Serialize, ::serde::Deserialize;
        Eqs = ::std::cmp::PartialEq, ::std::cmp::Eq;
    }
}

/*

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn duration_ms(_item: TokenStream) -> TokenStream {
    r#"#[serde_as(as = "DurationMilliSeconds<u64>")]"#.parse().unwrap()
}
*/
