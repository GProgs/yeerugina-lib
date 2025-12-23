use std::time::Duration;

use derive_aliases::derive;
use derive_more::{AsRef, Debug};
use rgb::RGB8;

/// The shortest smooth transition supported by the lamp.
///
/// Note that it is possible to instantiate Effects with durations less than this.
/// However, when Commands using these get passed to the Lamp, the short durations
/// will get clamped to be equal to MIN_DURATION.
pub const MIN_DURATION: Duration = Duration::from_millis(30);

/// Minimum value for brightness.
pub const MIN_BRIGHT: u8 = 1;
/// Maximum value for brightness.
pub const MAX_BRIGHT: u8 = 100;

/// Lowest allowed color brightness value (in kelvins).
pub const MIN_CT: u16 = 1700;
/// Highest allowed color brightness value (in kelvins).
pub const MAX_CT: u16 = 6500;

/// Newtype struct containing a duration limited to being larger than 30 ms.
#[derive(AsRef, Clone, Copy, Debug, ..Serde)]
#[serde(transparent)]
pub struct LimDuration(Duration);

/// Newtype struct representing a valid brightness value.
#[derive(AsRef, Clone, Copy, Debug, ..Serde)]
#[serde(transparent)]
pub struct Brightness(u8);

/// Newtype struct representing a valid color temperature value.
#[derive(AsRef, Clone, Copy, Debug, ..Serde)]
#[serde(transparent)]
pub struct ColorTemp(u16);

/// Newtype struct representing a valid RGB integer.
///
/// This refers to the int that is sent to the lamp.
#[derive(AsRef, Clone, Copy, Debug, ..Serde)]
#[serde(transparent)]
pub struct RGBInt(u32);

impl<T: Into<Duration>> From<T> for LimDuration {
    fn from(value: T) -> Self {
        Self(MIN_DURATION.max(value.into()))
    }
}

impl<T: Into<u8>> From<T> for Brightness {
    fn from(value: T) -> Self {
        Self(value.into().clamp(MIN_BRIGHT, MAX_BRIGHT))
    }
}

impl<T: Into<u16>> From<T> for ColorTemp {
    fn from(value: T) -> Self {
        Self(value.into().clamp(MIN_CT, MAX_CT))
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
