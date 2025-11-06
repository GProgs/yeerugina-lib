use log::info;
use std::{default, time::Duration};

pub struct Command {
    pub data: Value,
    pub eff: Effect,
    pub id: u8,
}

pub struct Value(InnerValue);

#[derive(strum_macros::EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(CommandKind))]
enum InnerValue {
    SetCtAbx(u16),
    SetRgb(u32),
}

impl Value {
    pub fn new_ct(ct: u16) -> Option<Self> {
        if !(1700..=6500).contains(&ct) {
            return None;
        }
        Some(Self(InnerValue::SetCtAbx(ct)))
    }

    pub fn new_rgb(rgb: u32) -> Option<Self> {
        if !(0..=0xFFFFFF).contains(&rgb) {
            return None;
        }
        Some(Self(InnerValue::SetRgb(rgb)))
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
    pub fn to_request(&self) -> String {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, derive_more::Display)]
pub struct Effect(InnerEffect);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum_macros::Display)]
enum InnerEffect {
    #[default]
    #[strum(to_string = "\"sudden\"")]
    Sudden,
    #[strum(to_string = "\"smooth\",todo")]
    Smooth(Duration),
}

impl Effect {
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

    pub fn new_sudden() -> Self {
        Effect(InnerEffect::Sudden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
