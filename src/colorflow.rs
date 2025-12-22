use std::{fmt::Display, time::Duration};

use derive_aliases::derive;
use derive_more::Debug;
use log::debug;
use rgb::RGB8;

use crate::{clamp_brightness, clamp_colortemp, clamp_duration};

#[derive(Debug, ..Eqs, ..Serde)]
/// An enum containing the data needed for a flow tuple.
enum CfExpression {
    /// Set the color to some value.
    ///
    /// Duration, color, brightness.
    Color(Duration, u32, u8),
    /// Set the color temperature.
    ///
    /// Duration, color temp, brightness.
    Ct(Duration, u16, u8),
    /// Sleep (i.e. don't do anything).
    Sleep(Duration),
}

/// Newtype struct containing the flow tuple (duration, mode, value, brightness).
///
/// Use the constructor methods, such as FlowTuple::new_ct() or Method::new_color()
/// to create instances of FlowTuple.
///
/// FlowTuple::default() will return a sleep of one second.
#[derive(Debug, Default, ..Eqs, ..Serde)]
#[serde(transparent)]
pub struct FlowTuple(CfExpression);

/// Newtype struct containing a Vector of flow tuples.
///
/// Provided for convenience.
#[derive(Debug, Default, ..Serde)]
#[serde(transparent)]
pub struct ColorFlow(pub Vec<FlowTuple>);

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
        (dur_clamp, bright_clamp)
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

impl Display for FlowTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f) // delegate to contained value
    }
}

impl Display for ColorFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cf_string = self
            .0
            .iter()
            .map(FlowTuple::to_string)
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{}", cf_string)
    }
}
