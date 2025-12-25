/*
macro_rules! cmd {
    (set_ct_abx, $id:expr, $dur:expr?) => {
        todo!()
    };
}
*/
/*
/// Macro that delegates the implementation of Display for newtypes
macro_rules! impl_display {
    ($i:ident) => {
        impl Display for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
    ($h:ident,$($t:ident),+) => {
        impl_display!($h);
        impl_display!($($t),+);
    };
}
impl_display!(Brightness, ColorTemp, RGBInt);
*/

/// Convenience macro to instantiate new Methods.
///
/// This is used by the impl block of Method,
/// more specifically the constructors, such as
/// new_set_ct_abx and new_set_hsv.
///
/// new_method!(effect) returns a tuple with the EffectKind and the associated duration.
/// new_method!(variant, effect; value) creates a Method containing the value.
/// new_method!(variant, effect; value1, value2) is the same but for a two-value Method.
#[macro_export]
macro_rules! new_method {
    ($e:ident) => {
        ($e.discriminant(), $e.get_duration().unwrap_or_default())
    };
    ($v:ident,$e:ident;$a:ident) => {{
        let (kind, dur) = new_method!($e);
        Method(MethodTuple::$v($a, kind, dur))
    }};
    ($v:ident,$e:ident;$a:ident,$b:ident) => {{
        let (kind, dur) = new_method!($e);
        Method(MethodTuple::$v($a, $b, kind, dur))
    }};
}

// tf = type from, tt = type this

/// Macro that creates an implementation of From for a newtype struct.
///
/// Used by crate::lim for its newtype structs.
///
/// Example invocation: impl_from_lim!(u8,Brightness,MIN_BRIGHT,MAX_BRIGHT)
#[macro_export]
macro_rules! impl_from_lim {
    ($tf:ty,$tt:ty,$min:expr,$max:expr) => {
        impl From<$tf> for $tt {
            fn from(value: $tf) -> Self {
                Self(value.clamp($min, $max))
            }
        }
    };
}
/// Macro that creates an implementation of Default for a newtype struct.
///
/// Used by crate::lim for its newtype structs.
///
/// Example invocation: impl_default!(Brightness,Self(MIN_BRIGHT))
#[macro_export]
macro_rules! impl_default {
    ($tt:ty,$exp:expr) => {
        impl Default for $tt {
            fn default() -> Self {
                $exp
            }
        }
    };
}
