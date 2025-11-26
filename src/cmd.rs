use std::time::Duration;

use derive_aliases::derive;
use derive_more::Debug;
use log::debug;
use rgb::RGB8;
use serde::{Deserialize, Serialize};
use serde_with::DurationMilliSeconds;
use serde_with::serde_as;
use strum::IntoDiscriminant;
use strum_macros::{Display, EnumDiscriminants};

/* A short word about aliases:
 * #[attr_alias(as_millis)] tells serde to represent Durations as milliseconds.
 * #[derive(..Serde)] derives Serialize and Deserialize.
 *
 * Enum discriminants need EXPLICIT derives - the alias won't work.
 *
 * If you want to represent a Duration as milliseconds, you will need to add attributes:
 * #[attr_alias::eval]
 * #[serde_as]
 */

const MIN_DURATION: Duration = Duration::from_millis(30);

/// Private enum containing the methods that are available.
///
/// This stores both the different methods "such as set_ct_abx"
/// as well as the parameters associated with each method.
#[attr_alias::eval]
#[serde_as]
#[derive(Debug, EnumDiscriminants, ..Serde)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
#[strum_discriminants(derive(Serialize, Deserialize))]
#[strum_discriminants(name(Method))] // don't use default name
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(doc = "The different kinds of commands that can be given to the lamp.")]
pub(self) enum MethodInner {
    SetCtAbx(u16, Effect, #[attr_alias(as_millis)] Duration),
    SetRgb(u32, Effect, #[attr_alias(as_millis)] Duration),
    SetHsv(u16, u8, Effect, #[attr_alias(as_millis)] Duration),
}

/// Newtype struct containing the data of the command (method + parameters)
#[derive(Debug, ..Serde)]
#[serde(transparent)]
pub struct MethodData(MethodInner);

#[attr_alias::eval]
#[serde_as]
#[derive(Debug, Default, EnumDiscriminants, ..Eqs, ..Serde)]
#[serde(rename_all = "snake_case")]
#[strum_discriminants(derive(Display, Serialize, Deserialize))]
#[strum_discriminants(name(Effect))] // don't use default name
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(vis(pub))]
pub enum EffectAndDuration {
    #[default]
    Sudden,
    Smooth(#[attr_alias(as_millis)] Duration),
}

#[derive(Debug, ..Serde)]
pub struct Command {
    id: u8,
    method: Method,
    params: MethodData,
}

// The idea is that we enforce limits in the constructors.
impl MethodData {
    pub fn new_set_ct_abx(ct: u16, data: &EffectAndDuration) -> Self {
        let ct_clamp = ct.clamp(1700, 6500);
        if ct != ct_clamp {
            debug!("MethodData | Color temperature was clamped");
        }
        Self(MethodInner::SetCtAbx(
            ct_clamp,
            Effect::from(data),
            data.get_dur(),
        ))
    }

    pub fn new_set_rgb<T: Into<RGB8>>(color: T, data: &EffectAndDuration) -> Self {
        let RGB8 { r, g, b } = color.into();
        let rgb_int = u32::from_be_bytes([0u8, r, g, b]);
        Self(MethodInner::SetRgb(
            rgb_int,
            Effect::from(data),
            data.get_dur(),
        ))
    }
}

impl EffectAndDuration {
    // Here, we enforce the requirement that durations must be more than 30ms.
    pub fn get_dur(&self) -> Duration {
        match self {
            Self::Sudden => Duration::ZERO,
            Self::Smooth(dur) => *dur.max(&MIN_DURATION),
        }
    }
}

impl Command {
    pub fn new(id: u8, params: MethodData) -> Self {
        // Enforce coherence through the constructor
        Self {
            id,
            method: params.0.discriminant(), //Method::from(&params.0),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn create_smooth_zero_secs() {
        let result = EffectAndDuration::Smooth(Duration::from_secs(0));
        let result_dur = result.get_dur();
        assert_eq!(result.discriminant(), Effect::Smooth);
        assert_ne!(result.discriminant(), Effect::Sudden);
        assert_eq!(result_dur, MIN_DURATION);
        assert_ne!(result_dur, Duration::ZERO);
    }
}
