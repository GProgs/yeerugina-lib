use std::{error::Error, time::Duration};
use yeerugina_lib::cmd::{Command, Effect, Method};

fn main() -> Result<(), Box<dyn Error>> {
    let id = 30u8;
    let eff = Effect::Smooth(Duration::from_millis(1500).into());
    let method = Method::new_rgb_from_parts(16, 32, 64);
    let cmd = Command { method, eff, id };

    println!("eff.to_string {}", eff.to_string());

    println!("cmd json {}", serde_json::to_string(&cmd)?);

    Ok(())
}
