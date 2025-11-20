use std::{error::Error, time::Duration};
use yeerugina_lib::{
    cmd::{Command, Effect, Method},
    cmd_new::{CommandData, CommandNew, EffectData},
};

fn main() -> Result<(), Box<dyn Error>> {
    let id = 30u8;
    let eff = Effect::Smooth(Duration::from_millis(1500).into());
    let method = Method::new_rgb_from_parts(16, 32, 64);
    let cmd = Command { method, eff, id };

    println!("eff.to_string {}", eff.to_string());

    println!("cmd json {}", serde_json::to_string(&cmd)?);

    // New stuff!

    let eff = EffectData::Smooth(Duration::from_millis(1500));
    let cmd_dat = CommandData::new_set_ct_abx(3600, &eff);
    let cmd = CommandNew::new(id, cmd_dat);

    println!("new cmd debug {:?}", cmd);
    println!("new cmd json {}", serde_json::to_string(&cmd)?);

    Ok(())
}
