#![allow(missing_docs, unused_crate_dependencies, unused_extern_crates)]
use std::{error::Error, time::Duration};
use yeerugina_lib::cmd::{Command, EffectAndDuration, MethodData};

fn main() -> Result<(), Box<dyn Error>> {
    let id = 30u8;
    let eff = EffectAndDuration::Smooth(Duration::from_millis(1500));
    let cmd_dat = MethodData::new_set_ct_abx(3600, &eff);
    let cmd = Command::new(id, cmd_dat);

    println!("new cmd debug {:?}", cmd);
    println!("new cmd json {}", serde_json::to_string(&cmd)?);

    Ok(())
}
