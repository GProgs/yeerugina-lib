use std::{fmt::Display, time::Duration};

use crate::lim::LimDuration;
use derive_aliases::derive;
use derive_more::Debug;
use log::debug;
use rgb::RGB8;

use crate::{clamp_brightness, clamp_colortemp};

#[derive(Debug, ..Eqs, ..Serde)]
/// An enum containing the data needed for a flow tuple.
enum CfExpression {
    /// Set the color to some value.
    ///
    /// Duration, color, brightness.
    Color(LimDuration, u32, u8),
    /// Set the color temperature.
    ///
    /// Duration, color temp, brightness.
    Ct(LimDuration, u16, u8),
    /// Sleep (i.e. don't do anything).
    Sleep(LimDuration),
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
    #[deprecated]
    fn clamp_dur_bright(dur: Duration, bright: u8) -> (Duration, u8) {
        unimplemented!();
    }

    /// Create a new flow tuple that indicates a change to a certain RGB color and brightness.
    pub fn new_color<T: Into<RGB8>>(dur: LimDuration, color: T, bright: u8) -> Self {
        let RGB8 { r, g, b } = color.into();
        let rgb_int = u32::from_be_bytes([0u8, r, g, b]);
        Self(CfExpression::Color(dur, rgb_int, bright))
    }

    /// Create a new flow tuple that indicates a change to a certain color temperature and brightness.
    pub fn new_ct(dur: LimDuration, ct: u16, bright: u8) -> Self {
        let ct_clamp = clamp_colortemp(ct);
        if ct != ct_clamp {
            debug!("FlowTuple | Color temperature was clamped");
        }
        Self(CfExpression::Ct(dur, ct_clamp, bright))
    }

    /// Create a new flow tuple that holds the current state of the lamp for some duration.
    pub fn new_sleep(dur: LimDuration) -> Self {
        Self(CfExpression::Sleep(dur))
    }
}

impl Default for CfExpression {
    fn default() -> Self {
        Self::Sleep(crate::lim::SECOND)
    }
}

impl Display for CfExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color(dur, col, bri) => {
                write!(f, "{},1,{},{}", dur.as_ref().as_millis(), col, bri)
            }
            Self::Ct(dur, ct, bri) => write!(f, "{},2,{},{}", dur.as_ref().as_millis(), ct, bri),
            Self::Sleep(dur) => write!(f, "{},7,0,0", dur.as_ref().as_millis()),
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
