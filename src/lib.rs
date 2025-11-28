//! Library for controlling Yeelight lamps through Rust.

// Annoying bug with the unused_crate_dependencies lint:
// rustc doesn't see all uses of extern crates,
// so stuff like "derive_more" flies under the radar,
// and the linter cries wolf.
// https://github.com/rust-lang/rust/issues/95513

// This is a VERY hacky workaround.
#![allow(unused_crate_dependencies)] // Allow in general
#![cfg(not(debug_assertions))] // ... but if we're not debugging ...
#![deny(unused_crate_dependencies)] // ... make sure we don't have any unused deps!
mod _unused {} // limit the scope of the cfg(not(...))

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
