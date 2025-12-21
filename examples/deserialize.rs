#![allow(missing_docs, unused_crate_dependencies, unused_extern_crates)]
use std::{error::Error, time::Duration};
use yeerugina_lib::cmd::{
    Command, Effect, Method,
    aux::{PowerMode, PowerOnOff},
};

fn main() -> Result<(), Box<dyn Error>> {
    let id = 30u8;
    let eff = Effect::Smooth(Duration::from_millis(1500));
    let cmd_dat = Method::new_set_ct_abx(3600, &eff);
    let cmd = Command::new(id, cmd_dat);

    println!("new cmd debug {:?}", cmd);
    println!("new cmd json {}", serde_json::to_string(&cmd)?);

    let eff_zero = Effect::Smooth(Duration::from_secs(0));
    let cmd2_dat = Method::new_set_ct_abx(4200, &eff_zero);
    let cmd2 = Command::new(id, cmd2_dat);

    println!("zero command json {}", serde_json::to_string(&cmd2)?);

    let toggler = Command::new(63, Method::new_toggle());
    println!("Toggle with {}", serde_json::to_string(&toggler)?);

    let no_mode = Command::new(
        11,
        Method::new_set_power(PowerOnOff::from(true), &eff, Some(PowerMode::ColorFlow)),
    );
    println!(
        "This won't have a mode: {}",
        serde_json::to_string(&no_mode)?
    );

    Ok(())
}
