//! Library for controlling Yeelight lamps through Rust.

#![feature(duration_constants)]

/// Modules for commands.
pub mod cmd;
/// Module for color flows.
pub mod colorflow;
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

/// The shortest smooth transition supported by the lamp.
///
/// Note that it is possible to instantiate Effects with durations less than this.
/// However, when Commands using these get passed to the Lamp, the short durations
/// will get clamped to be equal to MIN_DURATION.
pub const MIN_DURATION: Duration = Duration::from_millis(30);

/// Clamp a Duration to be more than 30 milliseconds.
pub fn clamp_duration(dur: Duration) -> Duration {
    dur.max(MIN_DURATION)
}

/// Clamp a brightness value to the interval 1..=100.
pub fn clamp_brightness(bright: u8) -> u8 {
    bright.clamp(1, 100)
}

/// Clamp a color temperature to the interval 1700..=6500 K.
pub fn clamp_colortemp(ct: u16) -> u16 {
    ct.clamp(1700, 6500)
}

// Add "unused" extern crates here.
// Annoying bug with the unused_crate_dependencies lint:
// rustc doesn't see all uses of extern crates,
// so stuff like "derive_more" flies under the radar,
// and the linter cries wolf.
// https://github.com/rust-lang/rust/issues/95513

use std::time::Duration;

// Put dev-dependencies emitting warns here.
#[cfg(test)]
use colog as _;
