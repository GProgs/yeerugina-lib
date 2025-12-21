use std::{fmt::Display, time::Duration};

use derive_aliases::derive;
use derive_more::Debug;
use log::debug;
use rgb::RGB8;
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::EnumString;

use crate::{clamp_brightness, clamp_colortemp, clamp_duration};

#[derive(Clone, Copy, Debug, Default, EnumString, ..Eqs, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
/// This enum defines the variable that is changed by a flow tuple.
///
/// A flow tuple can change the color or color temperature of the lamp.
/// Setting the brightness is included in every flow tuple.
pub enum CfMode {
    /// Set the lamp to have a certain color.
    Color = 1,
    /// Set the lamp to have a certain color temperature.
    Ct = 2,
    /// Do not do anything (i.e. keep the current state).
    ///
    /// This can be thought of as a NOP instruction.
    #[default]
    Sleep = 7,
}

#[derive(Debug, ..Eqs, ..Serde)]
pub(self) enum CfExpression {
    Color(Duration, u32, u8),
    Ct(Duration, u16, u8),
    Sleep(Duration),
}

/// Newtype struct containing the flow tuple (duration, mode, value, brightness).
///
/// Use the constructor methods, such as FlowTuple::new_ct() or Method::new_color()
/// to create instances of FlowTuple.
///
/// FlowTuple::default() will return a sleep of one second.
#[derive(Debug, Default, ..Eqs, ..Serde)]
pub struct FlowTuple(CfExpression);

impl FlowTuple {
    /// Clamp the duration and brightness to appropriate values.
    ///
    /// This is basically a wrapper that deals with logging.
    fn clamp_dur_bright(dur: Duration, bright: u8) -> (Duration, u8) {
        let (dur_clamp, bright_clamp) = (clamp_duration(dur), clamp_brightness(bright));
        if dur != dur_clamp {
            debug!("FlowTuple | Duration was clamped");
        }
        if bright != bright_clamp {
            debug!("FlowTuple | Brightness was clamped");
        }
        return (dur_clamp, bright_clamp);
    }

    /// Create a new flow tuple that indicates a change to a certain RGB color and brightness.
    pub fn new_color<T: Into<RGB8>>(dur: Duration, color: T, bright: u8) -> Self {
        let (dur_clamp, bright_clamp) = Self::clamp_dur_bright(dur, bright);
        let RGB8 { r, g, b } = color.into();
        let rgb_int = u32::from_be_bytes([0u8, r, g, b]);
        Self(CfExpression::Color(dur_clamp, rgb_int, bright_clamp))
    }

    /// Create a new flow tuple that indicates a change to a certain color temperature and brightness.
    pub fn new_ct(dur: Duration, ct: u16, bright: u8) -> Self {
        let (dur_clamp, bright_clamp) = Self::clamp_dur_bright(dur, bright);
        let ct_clamp = clamp_colortemp(ct);
        if ct != ct_clamp {
            debug!("FlowTuple | Color temperature was clamped");
        }
        Self(CfExpression::Ct(dur_clamp, ct_clamp, bright_clamp))
    }

    /// Create a new flow tuple that holds the current state of the lamp for some duration.
    pub fn new_sleep(dur: Duration) -> Self {
        let dur_clamp = clamp_duration(dur);
        if dur != dur_clamp {
            debug!("FlowTuple | Duration was clamped");
        }
        Self(CfExpression::Sleep(dur_clamp))
    }
}

impl Default for CfExpression {
    fn default() -> Self {
        // Requires nightly and duration_constants
        Self::Sleep(Duration::SECOND)
    }
}

impl Display for CfExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color(dur, col, bri) => write!(f, "{},1,{},{}", dur.as_millis(), col, bri),
            Self::Ct(dur, ct, bri) => write!(f, "{},2,{},{}", dur.as_millis(), ct, bri),
            Self::Sleep(dur) => write!(f, "{},7,0,0", dur.as_millis()),
        }
    }
}
