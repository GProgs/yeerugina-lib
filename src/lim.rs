use std::fmt::Display;
use std::time::Duration;

use derive_aliases::derive;
use derive_more::{AsRef, Debug, Display};
use rgb::RGB8;
use serde_with::DurationMilliSeconds;
use serde_with::serde_as;

use crate::impl_default;
use crate::impl_from_lim;

/// The shortest smooth transition supported by the lamp.
///
/// Note that it is possible to instantiate Effects with durations less than this.
/// However, when Commands using these get passed to the Lamp, the short durations
/// will get clamped to be equal to MIN_DURATION.
pub const MIN_DURATION: Duration = Duration::from_millis(30);
/// Equivalent to MIN_DURATION but as a LimDuration.
pub const MIN_LIMDURATION: LimDuration = LimDuration(MIN_DURATION);
/// A LimDuration equivalent to one second.
pub const LIM_SECOND: LimDuration = LimDuration(Duration::from_secs(1));

/// Minimum value for brightness.
pub const MIN_BRIGHT: u8 = 1;
/// Maximum value for brightness.
pub const MAX_BRIGHT: u8 = 100;

/// Lowest allowed color brightness value (in kelvins).
pub const MIN_CT: u16 = 1700;
/// Highest allowed color brightness value (in kelvins).
pub const MAX_CT: u16 = 6500;

/// Minimum value for HSV hue.
pub const MIN_HUE: u16 = 0;
/// Maximum value for HSV hue.
pub const MAX_HUE: u16 = 359;

/// Minimum value for HSV saturation.
pub const MIN_SAT: u8 = 0;
/// Maximum value for HSV saturation.
pub const MAX_SAT: u8 = 99;

// Guidelines for newtype structs with limits:
// Add #[serde(transparent)]
//
// Derive: AsRef and other common traits
// Impl: From, if possible Default
// Derive or impl: Display

/// Newtype struct containing a duration limited to being larger than 30 ms.
///
/// Note that LimDuration::default() will have a ZERO duration!
#[serde_as]
#[derive(AsRef, Clone, Copy, Debug, Default, ..Eqs, ..Serde)]
#[serde(transparent)]
pub struct LimDuration(#[serde_as(as = "DurationMilliSeconds<u64>")] Duration);

// Small note: Display for CfExpression needs our newtypes to implement Display.

/// Newtype struct representing a valid brightness value.
#[derive(AsRef, Clone, Copy, Debug, Display, ..Eqs, ..Serde)]
#[display("{_0}")]
#[serde(transparent)]
pub struct Brightness(u8);

/// Newtype struct representing a valid color temperature value.
#[derive(AsRef, Clone, Copy, Debug, Display, ..Eqs, ..Serde)]
#[display("{_0}")]
#[serde(transparent)]
pub struct ColorTemp(u16);

/// Newtype struct representing a valid RGB integer.
///
/// This refers to the int that is sent to the lamp.
#[derive(AsRef, Clone, Copy, Debug, Display, ..Eqs, ..Serde)]
#[display("{_0}")]
#[serde(transparent)]
pub struct RGBInt(u32);

// TODO create hue and saturation structs
/// Newtype struct representing a valid hue value.
#[derive(AsRef, Clone, Copy, Debug, Display, ..Eqs, ..Serde)]
#[display("{_0}")]
#[serde(transparent)]
pub struct Hue(u16);

/// Newtype struct representing a valid saturation value.
#[derive(AsRef, Clone, Copy, Debug, Display, ..Eqs, ..Serde)]
#[display("{_0}")]
#[serde(transparent)]
pub struct Sat(u8);

impl RGBInt {
    /// Constructs an RGBInt in native endian from a byte array in big endian.
    pub const fn from_be_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }
}

// Implementations of Default
// Most of these should be able to be done with impl_default!()
impl_default!(Brightness, Self(MAX_BRIGHT));
impl_default!(ColorTemp, Self(MAX_CT));
impl_default!(RGBInt, Self::from(0xFFFFFFu32));
impl_default!(Hue, Self(MIN_HUE));
impl_default!(Sat, Self(MAX_SAT));

// Note that since this impl is used by CfExpression,
// LimDuration will show the duration AS MILLISECONDS!
impl Display for LimDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_millis())
    }
}

// Implementations of From
// Note that more complicated expressions need to be written MANUALLY.
// impl_from_lim!() works ONLY for clamping.
impl From<Duration> for LimDuration {
    fn from(value: Duration) -> Self {
        Self(MIN_DURATION.max(value))
    }
}

impl_from_lim!(u8, Brightness, MIN_BRIGHT, MAX_BRIGHT);
impl_from_lim!(u16, ColorTemp, MIN_CT, MAX_CT);

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

impl_from_lim!(u16, Hue, MIN_HUE, MAX_HUE);
impl_from_lim!(u8, Sat, MIN_SAT, MAX_SAT);
