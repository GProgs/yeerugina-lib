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

use crate::clamp_brightness;
use crate::clamp_colortemp;
use crate::clamp_duration;

/*
 * Please follow this order:
 * - traits
 * - structs/enums, ordered s.t. dependencies are above dependents
 * (so newtypes come AFTER the types they enclose)
 * - impls (e.g. impl Command)
 * - impl _ for _ (like Display, From<T>,...)
 *
 * Please add #[serde(transparent)] to newtype structs.
 */

/* A short word about aliases:
 * #[attr_alias(ms)] tells serde to represent Durations as milliseconds.
 * #[derive(..Serde)] derives Serialize and Deserialize.
 *
 * Enum discriminants need EXPLICIT derives - the alias won't work.
 *
 * If you want to represent a Duration as milliseconds,
 * you will need to add these attributes to the enum:
 * #[attr_alias::eval]
 * #[serde_as]
 * and add #[attr_alias(ms)] to the field you're dealing with.
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
    /// Set the brightness of the lamp to a percentage in 1..=100.
    SetBright(u8, EffectKind, #[attr_alias(ms)] Duration),
    /// Power the lamp on/off.
    SetPower(
        aux::PowerOnOff,
        EffectKind,
        #[attr_alias(ms)] Duration,
        #[serde(skip_serializing_if = "Option::is_none")] Option<aux::PowerMode>,
    ),
    /// Toggle the lamp on/off.
    ///
    /// If the lamp is on, this turns it off, and vice versa.
    Toggle(),
    /// Apply a certain state to the lamp, turning it on if needed.
    SetScene(aux::SceneClass),
}
// attr_alias and serde_as: explained above.
// serde: renaming s.t. we get "set_ct_abx" etc.
// and untagged s.t. we get the correct JSON representation.
// strum_discriminants forward attributes to the MethodKind enum
// (which comes from deriving strum_macros::EnumDiscriminants)

/// Newtype struct containing the data of the command (method + parameters).
///
/// Use the constructor methods, such as Method::new_set_ct_abx() or Method::new_set_hsv()
/// to create instances of Method.
#[derive(Debug, ..Serde)]
#[serde(transparent)]
pub struct Method(MethodTuple);
// serde(transparent) uses the serialization of MethodTuple
// so this is truly a wrapper.

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
// See explanation of MethodTuple.
// The sudden transition is selected as the default (by me).
// Here we derive equalities cos they make sense for effects.

/// Struct describing a command that can be passed to the lamp.
///
/// This contains the elements needed to construct the command string using serde's serialization capabilities.
/// Pass your id and Method (taking ownership) to the Command::new() constructor to instantiate a Command.
#[derive(Debug, ..Serde)]
pub struct Command {
    /// A (non)unique ID that identifies the command.
    ///
    /// When the lamp responds to a command, the same ID will be included in the response.
    /// This can be used to distinguish between multiple successive commands and their responses.
    pub id: u8,
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
    fn unzip_effect(usr_eff: &Effect) -> (EffectKind, Duration) {
        // Handle Smooth effects w/ zero duration
        if let Effect::Smooth(_dur) = usr_eff
            && _dur.is_zero()
        {
            return (EffectKind::Sudden, Duration::ZERO);
        }

        // Sudden + Smooth w/ non-zero durations
        (
            usr_eff.discriminant(),
            match usr_eff {
                Effect::Sudden => Duration::ZERO,
                Effect::Smooth(dur) => clamp_duration(*dur), //*dur.max(&MIN_DURATION),
            },
        )
    }

    // For the constructors, I recommend using Self::process_usr_eff()
    // to easily get the MethodKind and Duration
    // which you will need to pass to the inner MethodTuple.
    // Now, you may think that this is redundant when dealing with Sudden.
    // However, we need a placeholder, otherwise the lamp won't accept the cmd.

    /// Create a new Method that sets the color temperature of the lamp.
    pub fn new_set_ct_abx(ct: u16, eff: &Effect) -> Self {
        let (kind, dur) = Self::unzip_effect(eff);
        let ct_clamp = clamp_colortemp(ct); //ct.clamp(1700, 6500);
        if ct != ct_clamp {
            debug!("Method | Color temperature was clamped");
        }
        Self(MethodTuple::SetCtAbx(ct_clamp, kind, dur))
    }

    /// Create a new Method that sets the RGB color of the lamp.
    pub fn new_set_rgb<T: Into<RGB8>>(color: T, eff: &Effect) -> Self {
        let (kind, dur) = Self::unzip_effect(eff);
        let RGB8 { r, g, b } = color.into(); // w/e we have in, convert it to RGB8
        let rgb_int = u32::from_be_bytes([0u8, r, g, b]);
        Self(MethodTuple::SetRgb(rgb_int, kind, dur))
    }

    /// Create a new Method that sets the color of the lamp using the HSV system of colors.
    pub fn new_set_hsv<S, T>(color: Hsv<S, T>, eff: &Effect) -> Self
    where
        T: IntoStimulus<f32>,
        f32: FromAngle<T>,
    {
        let (kind, dur) = Self::unzip_effect(eff);
        // colorspace is w/e, but we want f32
        let color_hsv: Hsv<S, f32> = color.into_format();
        let Hsv {
            hue,
            saturation,
            value: _,
            standard: _,
        } = color_hsv;
        let scaled_hue: u16 = hue.into_positive_degrees() as u16; // as cast does flooring
        // Calculate the saturation as an int from 0 to 100:
        let min_sat = Hsv::<S, f32>::min_saturation();
        let scaled_sat: u8 =
            (100.0 * (saturation - min_sat) / (Hsv::<S, f32>::max_saturation() - min_sat)) as u8;
        Self(MethodTuple::SetHsv(scaled_hue, scaled_sat, kind, dur))
    }

    /// Create a new Method that sets the brightness of the lamp to some percentage between 1 % and 100 %.
    pub fn new_set_bright(bright: u8, eff: &Effect) -> Self {
        let (kind, dur) = Self::unzip_effect(eff);
        let bright = clamp_brightness(bright); //bright.clamp(1, 100);
        Self(MethodTuple::SetBright(bright, kind, dur))
    }

    /// Create a new Method that powers the lamp on or off.
    pub fn new_set_power(
        power: aux::PowerOnOff,
        eff: &Effect,
        mode: Option<aux::PowerMode>,
    ) -> Self {
        let (kind, dur) = Self::unzip_effect(eff);
        Self(MethodTuple::SetPower(power, kind, dur, mode))
    }

    /// Create a new Method that turns the lamp on if it was previously off, and vice versa.
    pub fn new_toggle() -> Self {
        Self(MethodTuple::Toggle())
    }

    /// Create a new Method that sets the lamp to a certain state.
    pub fn new_set_scene(_class: aux::SceneClass) -> Self {
        todo!()
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

/// Module containing auxiliary variables.
///
/// These include such variables as:
/// - modes for the set_power command
/// - scene classes for the set_scene command
/// - adjustments for the set_adjust command
pub mod aux {
    use derive_aliases::derive;
    use derive_more::Debug;
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};
    use strum_macros::{Display, EnumDiscriminants, EnumString};

    /*
    If possible, please name the following objects as:
    (command)(variable), so for instance:
    PowerMode, SceneClass, AdjustAction, AdjustProp

    Please follow this order:
    - traits
    - structs/enums, ordered s.t. dependencies are above dependents
    (so newtypes come AFTER the types they enclose)
    When there are no dependencies, use the order of the commands
    as defined in the Yeelight specification.
    - impls (e.g. impl Command)
    - impl _ for _ (like Display, From<T>,...)

    Please add #[serde(transparent)] to newtype structs.
    */

    /// An enum describing the different modes the lamp can be set to upon powering it on.
    ///
    /// These are essentially just integers.
    #[derive(Clone, Copy, Debug, Default, EnumString, ..Eqs, Serialize_repr, Deserialize_repr)]
    #[repr(u8)]
    #[strum(ascii_case_insensitive)]
    pub enum PowerMode {
        /// Normal turn on operation.
        #[default]
        Normal = 0,
        /// Turn the lamp on and set it to display some color temperature.
        Ct = 1,
        /// Turn the lamp on to display an RGB color.
        Rgb = 2,
        /// Turn the lamp on to display a HSV color.
        Hsv = 3,
        /// Turn the lamp on and enable color flow mode.
        ColorFlow = 4,
        /// Turn the lamp on and set it to night light mode.
        ///
        /// Note that this mode is available only on ceiling lights.
        NightLight = 5,
    }

    #[derive(Clone, Copy, Debug, EnumString, ..Eqs, ..Serde)]
    #[serde(rename_all = "lowercase")]
    #[strum(ascii_case_insensitive)]
    /// The two power states: being on or off.
    pub enum PowerOnOff {
        /// The lamp is on.
        On,
        /// The lamp is off.
        Off,
    }

    #[derive(Clone, Copy, Debug, EnumDiscriminants, EnumString, ..Eqs, ..Serde)]
    #[serde(rename_all = "snake_case")]
    #[strum(ascii_case_insensitive)]
    #[strum_discriminants(derive(Display, Serialize, Deserialize))]
    #[strum_discriminants(name(SceneClass))] // don't use default name
    #[strum_discriminants(serde(rename_all = "snake_case"))]
    #[strum_discriminants(vis(pub))]
    #[strum_discriminants(doc = "The different kinds of scene classes.")]
    #[strum_discriminants(doc = "For example, Ct sets the color temperature of the lamp to some value.")]
    /// An enum containing the data that describes the change applied to the lamp.
    pub enum SceneTuple {
        /// Set the lamp to some RGB color and brightness.
        Color(u32, u8),
        /// Set the lamp to some HSV color and brightness.
        Hsv(u16, u8, u8),
        /// Set the lamp to some color temperature and brightness.
        Ct(u16, u8),
        //Cf(usize, todo!(), unimplemented!()), // We need a new struct for color flow action (0/1/2) and flow expression...
        // Vec<FlowExpression> where FlowExpression is an enum
        // maybe we could also use serde's flatten??
        // Either that or I'll write my own linked list / recursive struct:
        // struct ColorFlow {
        //      ... more stuff,
        //      #[serde(flatten)] // should work??
        //      next: Option<Box<Self>>,
        // }
        // Or make it even simpler: struct FlowExpression(dur,mode,val,bright)
        // Or just suck it and take a String, and provide a Factory(Vec<(...)>) so we can build them on our own.
        // Factory.add_ct(dur, ct, bright) would push a correct tuple to Factory.0
        // Maybe a newtype pub struct ColorFlow(String) that can be created ONLY from our factory could be an option??
        // like Factory.to_colorflow() -> ColorFlow
        /// Set the lamp to a certain brightness and turn it off after some minutes.
        AutoDelayOff(u8, usize),
    }

    #[derive(Clone, Copy, Debug, EnumString, ..Eqs, ..Serde)]
    #[serde(rename_all = "lowercase")]
    #[strum(ascii_case_insensitive)]
    /// The different ways a value may be adjusted (open-loop control).
    pub enum AdjustAction {
        /// Increase the specified value.
        Increase,
        /// Decrease the specified value.
        Decrease,
        /// Increase the specified value and wrap around upon reaching the maximum value.
        Circle,
    }

    #[derive(Clone, Copy, Debug, EnumString, ..Eqs, ..Serde)]
    #[serde(rename_all = "lowercase")]
    #[strum(ascii_case_insensitive)]
    /// The different values that can be adjusted.
    pub enum AdjustProp {
        /// The brightness of the lamp.
        Bright,
        /// The color temperature of the lamp.
        Ct,
        /// The hue of the lamp.
        ///
        /// Use only AdjustAction::Circle with this.
        Color,
    }

    impl From<bool> for PowerOnOff {
        fn from(value: bool) -> Self {
            match value {
                true => Self::On,
                false => Self::Off,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MIN_DURATION;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn create_smooth_zero_secs() {
        let result = Effect::Smooth(Duration::from_secs(0));
        let (_, result_dur) = Method::unzip_effect(&result);
        assert_eq!(result.discriminant(), EffectKind::Smooth);
        assert_ne!(result.discriminant(), EffectKind::Sudden);
        assert_eq!(result_dur, MIN_DURATION);
        assert_ne!(result_dur, Duration::ZERO);
    }
}
