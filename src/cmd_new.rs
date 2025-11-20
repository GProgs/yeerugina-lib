use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::DurationMilliSeconds;
use serde_with::serde_as;
use strum_macros::{Display, EnumDiscriminants};

const MIN_DURATION: Duration = Duration::from_millis(30);

/// Private enum containing the methods that are available.
///
/// This stores both the
#[serde_as]
#[derive(Debug, EnumDiscriminants, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
#[strum_discriminants(derive(Serialize, Deserialize))]
#[strum_discriminants(name(Method))] // don't use default name
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(doc = "The different kinds of commands that can be given to the lamp.")]
pub(self) enum MethodInner {
    SetCtAbx(
        u16,
        Effect,
        // Please consider writing your OWN proc_macro to eliminate THIS monstrosity. TODO
        #[serde_as(as = "DurationMilliSeconds<u64>")] Duration,
    ),
    SetRgb(u32, Effect, Duration),
}

/// Newtype struct containing the data of the command (method + parameters)
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandData(MethodInner);

#[derive(Debug, EnumDiscriminants, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum_discriminants(derive(Display, Serialize, Deserialize))]
#[strum_discriminants(name(Effect))] // don't use default name
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(vis(pub))]
pub enum EffectData {
    Sudden,
    Smooth(Duration),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandNew {
    id: u8,
    method: Method,
    params: CommandData,
}

impl CommandData {
    pub fn new_set_ct_abx(ct: u16, data: &EffectData) -> Self {
        Self(MethodInner::SetCtAbx(
            ct,
            Effect::from(data),
            data.get_dur(),
        ))
    }
}

impl EffectData {
    // Here, we enforce the requirement that durations must be more than 30ms.
    pub fn get_dur(&self) -> Duration {
        match self {
            Self::Sudden => Duration::ZERO,
            Self::Smooth(dur) => *dur.max(&MIN_DURATION),
        }
    }
}

impl CommandNew {
    pub fn new(id: u8, params: CommandData) -> Self {
        // Enforce coherence through the constructor
        Self {
            id,
            method: Method::from(&params.0),
            params,
        }
    }
}
