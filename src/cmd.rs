use derive_more::{Debug, Display};
use log::info;
use std::{fmt::Display, time::Duration};

/*
 * Please follow this order:
 * - structs/enums, ordered s.t. dependencies are above dependents
 * (so newtypes come AFTER the types they enclose)
 * - impls (e.g. impl Command)
 * - impl _ for _ (like Display, From<T>,...)
 */

#[derive(strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Display))]
#[strum_discriminants(name(CommandKind))] // don't use default name
#[strum_discriminants(vis(pub))]
#[strum_discriminants(doc = "The different kinds of commands that can be given to the lamp.")]
#[strum_discriminants(
    doc = "They are derived from the [InnerAction] enum, but do not contain any values, and are publically available."
)]
/// A change done to a lamp.
///
/// This is the inner enum of [Action]. The commands that can be given to the lamp are defined here.
/// The enum variants also contain data needed to accomplish these actions.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
//#[display("\"method\":{_variant}")]
enum InnerAction {
    /// Set the color temperature of the lamp to some number of kelvins.
    #[display("\"set_ct_abx\",\"params\":[{_0}")]
    SetCtAbx(#[debug("{_0}K")] u16), // add kelvin unit
    /// Set the lamp to display a color by passing a u32.
    /// The eight smallest bits denote the blue value, then the following bytes denote green and red.
    /// For example, in order to set the lamp to display a purple color (RGB 165,26,234), you can pass 0xa61aeau32.
    /// Generally, for a hex color #RRGGBB, you pass the integer 0x00{RR}{GG}{BB}.
    #[display("\"set_rgb\",\"params\":[{_0}")]
    SetRgb(#[debug("{_0:x}")] u32), // print as hex
}

/// The change that is done by a [Command].
///
/// This is a newtype struct enclosing an enum so that restrictions on values can be enforced.
#[derive(Clone, Copy, Display, Debug, PartialEq, Eq)]
pub struct Action(#[debug("{_0:?}")] InnerAction);
// remove prefix SmoothDuration() from Debug output

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// A newtype enclosing a [Duration].
///
/// This is used to enforce the requirement that smooth transitions must last at least 30 milliseconds.
/// The easiest way to create a SmoothDuration struct is to call SmoothDuration::from();
/// or alternatively, to call the into() method on a Duration.
pub struct SmoothDuration(Duration);

#[derive(Clone, Copy, Display, Debug, Default, PartialEq, Eq)]
/// The transition between the current and new state of the lamp.
///
/// In addition to constructing instances manually, Durations can be converted to [Effect](Effects)
/// using [Effect]::from() or the into() method on a Duration.
pub enum Effect {
    #[default]
    #[display("\"sudden\"")]
    /// Change the lamp to the new state immediately.
    Sudden,
    #[display("\"smooth\", {_0}")]
    /// Smoothly fade into the new state over some [SmoothDuration].
    Smooth(#[debug("{}ms",_0.0.as_millis())] SmoothDuration), // print as millis
}

/// A Yeelight command represented as a struct.
///
/// Assuming you have a valid [Action] and [Effect], you can construct the [Command] struct yourself.
/// What the command does is stored in the data field of [Command].
#[derive(Clone, Copy, Display, Debug)]
#[display(r#"{{"id":{id},"method":{action}, {eff}]}}\r\n"#)]
pub struct Command {
    /// This field denotes the change done by [Command], along with other data, such as color temperature or RGB value.
    pub action: Action,
    /// The transition (sudden or smooth) is represented by [Effect].
    pub eff: Effect,
    /// A integer used to distinguish between requests.
    pub id: u8,
}
/* Explanation for the display string:
 * Here, we do {"id":32,"method":
 * then action's Display does "set_ct_abx","params":[3200
 * then we add a comma and space ,
 * then effect's Display does "smooth", 3200
 * and we finish off with ]}\r\n
 */

impl Command {
    /// Get the String that should be sent to the lamp through a TcpStream in order to perform the [Command].
    ///
    /// Note that the terminator `\r\n` is not included in the output.
    pub fn to_request(&self) -> String {
        todo!()
    }
}

impl Action {
    /// Create a new Action for changing the color temperature of the lamp to some value.
    ///
    /// This method enforces the constraint 1700K <= ct <= 6500K.
    pub fn new_ct(ct: u16) -> Option<Self> {
        if !(1700..=6500).contains(&ct) {
            info!("Attempted to create SetCtAbx with {ct}K");
            return None;
        }
        Some(Self(InnerAction::SetCtAbx(ct)))
    }

    /// Create a new Action for changing the color of the lamp to some RGB color.
    ///
    /// The largest byte of the u32 will be ignored.
    pub fn new_rgb_from_int(rgb: u32) -> Option<Self> {
        if !(0..=0xFFFFFF).contains(&rgb) {
            info!("Discarding highest byte from SetRgb with {:#x}", rgb);
            Some(Self(InnerAction::SetRgb(rgb & 0x00FFFFFFu32)))
        } else {
            Some(Self(InnerAction::SetRgb(rgb)))
        }
    }

    /// Create a new Action for changing the color of the lamp to some RGB color.
    ///
    /// This function takes three u8 values representing the red, green, and blue channels.
    pub fn new_rgb_from_parts(r: u8, g: u8, b: u8) -> Option<Self> {
        let rgb = u32::from_be_bytes([0x0, r, g, b]);
        // We don't need to verify since we know that the largest byte is zero
        Some(Self(InnerAction::SetRgb(rgb)))
    }
}

impl Display for SmoothDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"smooth\",{}", self.0.as_millis())
    }
}

impl From<Duration> for SmoothDuration {
    fn from(value: Duration) -> Self {
        if value.as_millis() < 30 {
            Self(Duration::from_millis(30))
        } else {
            Self(value)
        }
    }
}

impl From<Duration> for Effect {
    fn from(value: Duration) -> Self {
        if value.is_zero() {
            Self::Sudden
        } else {
            Self::Smooth(SmoothDuration::from(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn create_smooth_zero_secs() {
        let result: Effect = Duration::from_secs(0).into();
        assert_eq!(result, Effect::Sudden);
    }

    #[test]
    fn create_smooth_zero_millis() {
        let result: Effect = Duration::from_millis(0).into();
        assert_eq!(result, Effect::Sudden);
    }

    #[test]
    fn create_smooth_short_1() {
        let result: Effect = Duration::from_millis(10).into();
        let expect: Effect = Duration::from_millis(30).into();
        assert_eq!(result, expect);
    }

    #[test]
    fn create_smooth_short_2() {
        let result: Effect = Duration::from_millis(25).into();
        let expect: Effect = Duration::from_millis(30).into();
        assert_eq!(result, expect);
    }

    #[test]
    fn create_smooth_long() {
        let result: Effect = Duration::from_millis(2000).into();
        let expect: Effect = Duration::from_secs(2).into();
        assert_eq!(result, expect);
    }

    #[test]
    fn into_smoothduration_short() {
        let smoothdur: SmoothDuration = Duration::from_millis(10).into();
        let result = Effect::Smooth(smoothdur);
        let expect: Effect = Duration::from_millis(30).into();
        assert_eq!(result, expect);
    }

    #[test]
    fn into_smoothduration_long() {
        let smoothdur: SmoothDuration = Duration::from_secs(3).into();
        let result = Effect::Smooth(smoothdur);
        let expect: Effect = Duration::from_millis(3000).into();
        assert_eq!(result, expect);
    }

    #[test]
    fn rgb_eq() {
        let rgb_1 = Action::new_rgb_from_int(0xDEADFEu32);
        let rgb_2 = Action::new_rgb_from_parts(222, 173, 254);
        assert_eq!(rgb_1, rgb_2);
    }

    #[test]
    fn rgb_eqne() {
        let rgb_1 = Action::new_rgb_from_int(0xA61A3Au32);
        let rgb_2_wrong = Action::new_rgb_from_parts(58, 26, 166);
        assert_ne!(rgb_1, rgb_2_wrong);
        let rgb_2_right = Action::new_rgb_from_parts(166, 26, 58);
        assert_ne!(rgb_2_wrong, rgb_2_right);
        assert_eq!(rgb_1, rgb_2_right);
    }
}
