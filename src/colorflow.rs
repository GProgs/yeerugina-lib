use std::fmt::Display;

use crate::lim::{Brightness, ColorTemp, LimDuration, RGBInt};
use derive_aliases::derive;
use derive_more::Debug;
use rgb::RGB8;

#[derive(Debug, ..Eqs, ..Serde)]
/// An enum containing the data needed for a flow tuple.
enum CfExpression {
    /// Set the color to some value.
    ///
    /// Duration, color, brightness.
    Color(LimDuration, RGBInt, Brightness),
    /// Set the color temperature.
    ///
    /// Duration, color temp, brightness.
    Ct(LimDuration, ColorTemp, Brightness),
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
    /// Create a new flow tuple that indicates a change to a certain RGB color and brightness.
    pub fn new_color<T: Into<RGB8>>(dur: LimDuration, color: T, bright: Brightness) -> Self {
        let RGB8 { r, g, b } = color.into();
        let rgb_int = RGBInt::from_be_bytes([0u8, r, g, b]);
        Self(CfExpression::Color(dur, rgb_int, bright))
    }

    /// Create a new flow tuple that indicates a change to a certain color temperature and brightness.
    pub fn new_ct(dur: LimDuration, ct: ColorTemp, bright: Brightness) -> Self {
        Self(CfExpression::Ct(dur, ct, bright))
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
