/*
macro_rules! cmd {
    (set_ct_abx, $id:expr, $dur:expr?) => {
        todo!()
    };
}
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
