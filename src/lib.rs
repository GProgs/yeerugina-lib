//! Library for controlling Yeelight lamps through Rust.
use palette as _;

/// Modules for commands.
#[allow(missing_docs)]
pub mod cmd;
/// Module for code related to interfacing with lamps.
pub mod lamp;

mod derive_alias {
    derive_aliases::define! {
        Serde = ::serde::Serialize, ::serde::Deserialize;
        Eqs = ::std::cmp::PartialEq, ::std::cmp::Eq;
    }
}

/*
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
*/
