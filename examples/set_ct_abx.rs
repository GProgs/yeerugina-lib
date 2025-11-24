#![allow(missing_docs, unused_crate_dependencies, unused_extern_crates)]
use smol::block_on;
use std::{error::Error, thread::sleep, time::Duration};
use yeerugina_lib::{
    cmd::{Command, EffectDuration, MethodData},
    lamp::{AsyncLamp, Lamp},
};

fn main() -> Result<(), Box<dyn Error>> {
    // Safe because this is a single-threaded program.
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    colog::init();
    let sleep_dur = Duration::from_secs(3);

    let id = 30u8;
    let eff = EffectDuration::Smooth(Duration::from_millis(1500));
    let cmd_dat = MethodData::new_set_ct_abx(3600, &eff);
    let cmd2_dat = MethodData::new_set_ct_abx(3000, &eff);
    let cmd = Command::new(id, cmd_dat);
    let cmd_2 = Command::new(id, cmd2_dat);

    {
        let mut lamp = Lamp::connect("192.168.1.3:55443")?;
        lamp.send_cmd(&cmd)?;
    }
    sleep(sleep_dur);
    {
        let mut lamp = block_on(async { AsyncLamp::connect("192.168.1.3:55443").await })?;
        block_on(async { lamp.send_cmd(&cmd_2).await })?;
    }
    sleep(sleep_dur);
    Ok(())
}
