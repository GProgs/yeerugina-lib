use std::time::Duration;

use derive_aliases::derive;
use derive_more::{AsRef, Debug};
use rgb::RGB8;
use serde_with::DurationMilliSeconds;
use serde_with::serde_as;

/// The shortest smooth transition supported by the lamp.
///
/// Note that it is possible to instantiate Effects with durations less than this.
/// However, when Commands using these get passed to the Lamp, the short durations
/// will get clamped to be equal to MIN_DURATION.
pub const MIN_DURATION: Duration = Duration::from_millis(30);
/// Equivalent to MIN_DURATION but as a LimDuration.
pub const MIN_LIMDURATION: LimDuration = LimDuration(MIN_DURATION);
/// A LimDuration equivalent to one second.
pub const SECOND: LimDuration = LimDuration(Duration::from_secs(1));

/// Minimum value for brightness.
pub const MIN_BRIGHT: u8 = 1;
/// Maximum value for brightness.
pub const MAX_BRIGHT: u8 = 100;

/// Lowest allowed color brightness value (in kelvins).
pub const MIN_CT: u16 = 1700;
/// Highest allowed color brightness value (in kelvins).
pub const MAX_CT: u16 = 6500;

/// Newtype struct containing a duration limited to being larger than 30 ms.
///
/// Note that LimDuration::default() will have a ZERO duration!
#[serde_as]
#[derive(AsRef, Clone, Copy, Debug, Default, ..Eqs, ..Serde)]
#[serde(transparent)]
pub struct LimDuration(#[serde_as(as = "DurationMilliSeconds<u64>")] Duration);

/// Newtype struct representing a valid brightness value.
#[derive(AsRef, Clone, Copy, Debug, ..Eqs, ..Serde)]
#[serde(transparent)]
pub struct Brightness(u8);

/// Newtype struct representing a valid color temperature value.
#[derive(AsRef, Clone, Copy, Debug, ..Eqs, ..Serde)]
#[serde(transparent)]
pub struct ColorTemp(u16);

/// Newtype struct representing a valid RGB integer.
///
/// This refers to the int that is sent to the lamp.
#[derive(AsRef, Clone, Copy, Debug, ..Eqs, ..Serde)]
#[serde(transparent)]
pub struct RGBInt(u32);

impl From<Duration> for LimDuration {
    fn from(value: Duration) -> Self {
        Self(MIN_DURATION.max(value))
    }
}

impl From<u8> for Brightness {
    fn from(value: u8) -> Self {
        Self(value.clamp(MIN_BRIGHT, MAX_BRIGHT))
    }
}

impl From<u16> for ColorTemp {
    fn from(value: u16) -> Self {
        Self(value.clamp(MIN_CT, MAX_CT))
    }
}

impl From<u32> for RGBInt {
    fn from(value: u32) -> Self {
        Self(value.min(0x00FFFFFF))
    }
}

impl From<RGB8> for RGBInt {
    fn from(value: RGB8) -> Self {
        let RGB8 { r, g, b } = value;
        let rgb_int = u32::from_be_bytes([0u8, r, g, b]);
        Self(rgb_int)
    }
}
