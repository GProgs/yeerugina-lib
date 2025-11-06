use log::info;
use std::{default, time::Duration};

/// A Yeelight command represented as a struct.
///
/// Assuming you have a valid [Action] and [Effect], you can construct the [Command] struct yourself.
/// What the command does is stored in the data field of [Command].
pub struct Command {
    /// This field denotes the change done by [Command], along with other data, such as color temperature or RGB value.
    pub action: Action,
    /// The transition (sudden or smooth) is represented by [Effect].
    pub eff: Effect,
    /// A integer used to distinguish between requests.
    pub id: u8,
}

/// The change that is done by a [Command].
///
/// This is a newtype struct enclosing an enum so that restrictions on values can be enforced.
pub struct Action(InnerAction);

///
///
///
//#[doc = "They are derived from the [InnerAction] enum, but do not contain any values, and are publically available."]
#[derive(strum_macros::EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(CommandKind))]
#[strum_discriminants(doc = "The different kinds of commands that can be given to the lamp.")]
/// A change done to a lamp.
///
/// This is the inner enum of [Action]. The commands that can be given to the lamp are defined here.
/// The enum variants also contain data needed to accomplish these actions.
enum InnerAction {
    /// Set the color temperature of the lamp to some number of kelvins.
    SetCtAbx(u16),
    /// Set the lamp to display a color by passing a u32.
    /// The eight smallest bits denote the blue value, then the following bytes denote green and red.
    /// For example, in order to set the lamp to display a purple color (RGB 128,49,181), you can pass 0x8031b5u32.
    /// Generally, for a hex color #RRGGBB, you pass the integer 0x00{RR}{GG}{BB}.
    SetRgb(u32), // TODO rewrite to use [u8; 3] maybe?
}

impl Action {
    /// Create a new Action for changing the color temperature of the lamp to some value.
    ///
    /// This method enforces the constraint 1700K <= ct <= 6500K.
    pub fn new_ct(ct: u16) -> Option<Self> {
        if !(1700..=6500).contains(&ct) {
            return None;
        }
        Some(Self(InnerAction::SetCtAbx(ct)))
    }

    /// Create a new Action for changing the color of the lamp to some RGB color.
    ///
    /// The two largest bytes of the u32 must be zero.
    pub fn new_rgb(rgb: u32) -> Option<Self> {
        if !(0..=0xFFFFFF).contains(&rgb) {
            return None;
        }
        Some(Self(InnerAction::SetRgb(rgb)))
    }

    /*
    pub fn new<T>(kind: CommandKind, data: T) -> Option<Self> {
        match kind {
            CommandKind::SetCtAbx => Self::new_ct(data),
            CommandKind::SetRgb => Self::new_rgb(data),
        }
    }
    */
}

impl Command {
    /// Get the String that should be sent to the lamp through a TcpStream in order to perform the [Command].
    ///
    /// Note that the terminator `\r\n` is not included in the output.
    pub fn to_request(&self) -> String {
        todo!()
    }
}

// TODO consider rewriting Effect as an enum, and having a SmoothDuration newtype that enforces the time constraint?

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, derive_more::Display)]
/// The transition between the current and new state of the lamp.
///
/// This is a newtype struct enclosing an enum so that restrictions on the smooth duration can be enforced.
pub struct Effect(InnerEffect);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum_macros::Display)]
/// A transition between lamp states.
///
/// This is the inner enum of [Effect].
enum InnerEffect {
    #[default]
    #[strum(to_string = "\"sudden\"")]
    /// Change the lamp to the new state instantly.
    Sudden,
    #[strum(to_string = "\"smooth\",todo")]
    /// Smoothly fade into the new state over some Duration.
    Smooth(Duration),
}

impl Effect {
    /// Create a new instance of a smooth transition.
    ///
    /// If the given Duration is zero, the effect will be converted to a sudden transition.
    /// If the given Duration is <=30ms, the Duration of the smooth effect will be clamped to 30ms.
    pub fn new_smooth(dur: Duration) -> Self {
        // Logic depending on the length of dur
        // Zero Durations converted to sudden transitions
        // <30ms clamped to 30ms
        match dur.as_millis() {
            _ if dur.is_zero() => Effect(InnerEffect::Sudden),
            0..=30 => {
                info!("Clamped effect duration");
                Effect(InnerEffect::Smooth(Duration::from_millis(30)))
            }
            _ => Effect(InnerEffect::Smooth(dur)),
        }
    }

    /// Create a new instance of a sudden transition.
    pub fn new_sudden() -> Self {
        Effect(InnerEffect::Sudden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn create_smooth_zero_secs() {
        let result = Effect::new_smooth(Duration::from_secs(0));
        assert_eq!(result, Effect(InnerEffect::Sudden));
    }

    #[test]
    fn create_smooth_zero_millis() {
        let result = Effect::new_smooth(Duration::from_millis(0));
        assert_eq!(result, Effect(InnerEffect::Sudden));
    }

    #[test]
    fn create_smooth_short_1() {
        let result = Effect::new_smooth(Duration::from_millis(10));
        let expect = Effect(InnerEffect::Smooth(Duration::from_millis(30)));
        assert_eq!(result, expect);
    }

    #[test]
    fn create_smooth_short_2() {
        let result = Effect::new_smooth(Duration::from_millis(25));
        let expect = Effect(InnerEffect::Smooth(Duration::from_millis(30)));
        assert_eq!(result, expect);
    }

    #[test]
    fn create_smooth_long() {
        let result = Effect::new_smooth(Duration::from_millis(2000));
        let expect = Effect(InnerEffect::Smooth(Duration::from_secs(2)));
        assert_eq!(result, expect);
    }
}
