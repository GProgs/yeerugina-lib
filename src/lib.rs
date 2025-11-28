//! Library for controlling Yeelight lamps through Rust.

/// Modules for commands.
pub mod cmd;
/// Module for code related to interfacing with lamps.
pub mod lamp;

/// Module containing aliases for derive macros.
///
/// This is used by the "derive_aliases" crate.
mod derive_alias {
    derive_aliases::define! {
        Serde = ::serde::Serialize, ::serde::Deserialize;
        Eqs = ::std::cmp::PartialEq, ::std::cmp::Eq;
    }
}

// Add "unused" extern crates here.
// Annoying bug with the unused_crate_dependencies lint:
// rustc doesn't see all uses of extern crates,
// so stuff like "derive_more" flies under the radar,
// and the linter cries wolf.
// https://github.com/rust-lang/rust/issues/95513

// Put dev-dependencies emitting warns here.
#[cfg(test)]
use colog as _;
