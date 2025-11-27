use std::time::Duration;

use derive_aliases::derive;
use derive_more::Debug;
use log::debug;
use palette::Hsv;
use palette::angle::FromAngle;
use palette::stimulus::IntoStimulus;
use rgb::RGB8;
use serde::{Deserialize, Serialize};
use serde_with::DurationMilliSeconds;
use serde_with::serde_as;
use strum::IntoDiscriminant;
use strum_macros::{Display, EnumDiscriminants};

/* A short word about aliases:
 * #[attr_alias(ms)] tells serde to represent Durations as milliseconds.
 * #[derive(..Serde)] derives Serialize and Deserialize.
 *
 * Enum discriminants need EXPLICIT derives - the alias won't work.
 *
 * If you want to represent a Duration as milliseconds, you will need to add attributes:
 * #[attr_alias::eval]
 * #[serde_as]
 */

const MIN_DURATION: Duration = Duration::from_millis(30);

/*
macro_rules! duration_ms {
    () => {
        #[attr_alias(as_millis)]
        Duration
    };
}
*/

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
#[strum_discriminants(name(MethodKind))] // don't use default name
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(doc = "The different kinds of commands that can be given to the lamp.")]
pub(self) enum MethodTuple {
    /// Set the color temperature of the lamp.
    SetCtAbx(u16, EffectKind, #[attr_alias(ms)] Duration),
    /// Set the RGB color of the lamp.
    ///
    /// Largest byte is zero, then comes the red channel, then green and blue.
    SetRgb(u32, EffectKind, #[attr_alias(ms)] Duration),
    /// Set the HSV color of the lamp.
    ///
    /// The hue is in 0..360, and saturation in 0..100.
    SetHsv(u16, u8, EffectKind, #[attr_alias(ms)] Duration),
}

/// Newtype struct containing the data of the command (method + parameters).
///
/// Use the constructor methods, such as Method::new_set_ct_abx() or Method::new_set_hsv()
/// to create instances of Method.
#[derive(Debug, ..Serde)]
#[serde(transparent)]
pub struct Method(MethodTuple);

/// Public enum describing the way a command is applied.
///
/// A sudden transition means that the lamp changes color / state instantly,
/// while a smooth transition means the lamp gradually fades into the new state.
/// A smooth transition must last for at least 30 milliseconds. Any Durations less than this
/// will get clamped to be equal to 30 ms. Passing a zero Duration makes this Effect
/// behave like a sudden transition.
#[attr_alias::eval]
#[serde_as]
#[derive(Debug, Default, EnumDiscriminants, ..Eqs, ..Serde)]
#[serde(rename_all = "snake_case")]
#[strum_discriminants(derive(Display, Serialize, Deserialize))]
#[strum_discriminants(name(EffectKind))] // don't use default name
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(doc = "The two possible effect types; smooth and sudden transition.")]
pub enum Effect {
    /// Changes the color/state of the lamp instantly.
    #[default]
    Sudden,
    /// Changes the color/state of the lamp gradually, over some time period.
    Smooth(#[attr_alias(ms)] Duration),
}

/// Struct describing a command that can be passed to the lamp.
///
/// This contains the elements needed to construct the command string using serde's serialization capabilities.
/// Pass your id and Method (taking ownership) to the Command::new() constructor to instantiate a Command.
#[derive(Debug, ..Serde)]
pub struct Command {
    id: u8,
    method: MethodKind,
    params: Method,
}

// The idea is that we enforce limits in the constructors.
impl Method {
    /// Get the effect that will be sent to the lamp.
    ///
    /// The idea here is that we "validate" the user input, and check whether the duration is zero.
    /// If it is, convert it to a sudden transition.
    /// For non-zero values, the duration gets clamped to min. 30 milliseconds.
    /// Sudden transitions get a zero Duration as a placeholder, which gets ignored by the lamp.
    fn process_usr_eff(usr_eff: &Effect) -> (EffectKind, Duration) {
        if let Effect::Smooth(_dur) = usr_eff
            && _dur.is_zero()
        {
            return (EffectKind::Sudden, Duration::ZERO);
        }

        (
            usr_eff.discriminant(),
            match usr_eff {
                Effect::Sudden => Duration::ZERO,
                Effect::Smooth(dur) => *dur.max(&MIN_DURATION),
            },
        )
    }

    /// Create a new Method that sets the color temperature of the lamp.
    pub fn new_set_ct_abx(ct: u16, eff: &Effect) -> Self {
        let (kind, dur) = Self::process_usr_eff(eff);
        let ct_clamp = ct.clamp(1700, 6500);
        if ct != ct_clamp {
            debug!("MethodData | Color temperature was clamped");
        }
        Self(MethodTuple::SetCtAbx(ct_clamp, kind, dur))
    }

    /// Create a new Method that sets the RGB color of the lamp.
    pub fn new_set_rgb<T: Into<RGB8>>(color: T, eff: &Effect) -> Self {
        let (kind, dur) = Self::process_usr_eff(eff);
        let RGB8 { r, g, b } = color.into();
        let rgb_int = u32::from_be_bytes([0u8, r, g, b]);
        Self(MethodTuple::SetRgb(rgb_int, kind, dur))
    }

    /// Create a new Method that sets the color of the lamp using the HSV system of colors.
    pub fn new_set_hsv<S, T>(color: Hsv<S, T>, eff: &Effect) -> Self
    where
        T: IntoStimulus<f32>,
        f32: FromAngle<T>,
    {
        let (kind, dur) = Self::process_usr_eff(eff);
        let color_hsv: Hsv<S, f32> = color.into_format();
        let Hsv {
            hue,
            saturation,
            value: _,
            standard: _,
        } = color_hsv;
        let scaled_hue: u16 = hue.into_positive_degrees() as u16; // as cast does flooring
        let min_sat = Hsv::<S, f32>::min_saturation();
        let scaled_sat: u8 =
            ((saturation - min_sat) / (Hsv::<S, f32>::max_saturation() - min_sat)) as u8;
        Self(MethodTuple::SetHsv(scaled_hue, scaled_sat, kind, dur))
    }
}

impl Command {
    /// Create a new Command by passing in the id and Method (i.e. change color temp or RGB color).
    pub fn new(id: u8, params: Method) -> Self {
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
        let result = Effect::Smooth(Duration::from_secs(0));
        let (_, result_dur) = Method::process_usr_eff(&result);
        assert_eq!(result.discriminant(), EffectKind::Smooth);
        assert_ne!(result.discriminant(), EffectKind::Sudden);
        assert_eq!(result_dur, MIN_DURATION);
        assert_ne!(result_dur, Duration::ZERO);
    }
}
