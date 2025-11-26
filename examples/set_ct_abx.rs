#![allow(missing_docs, unused_crate_dependencies, unused_extern_crates)]
use smol::block_on;
use std::{error::Error, thread::sleep, time::Duration};
use yeerugina_lib::{
    cmd::{Command, EffectAndDuration, MethodData},
    lamp::{AsyncLamp, Lamp},
};

fn main() -> Result<(), Box<dyn Error>> {
    // Safe because this is a single-threaded program.
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    colog::init();

    let id = 30u8;
    let eff = EffectAndDuration::Smooth(Duration::from_millis(1500));
    let cmd = Command::new(id, MethodData::new_set_ct_abx(3600, &eff));
    let cmd2 = Command::new(id, MethodData::new_set_ct_abx(3000, &eff));

    println!(
        "1st command {}",
        serde_json::to_string(&cmd).unwrap_or_default()
    );
    println!(
        "2nd command {}",
        serde_json::to_string(&cmd2).unwrap_or_default()
    );

    let sleep_dur = Duration::from_secs(3);
    {
        let mut lamp = Lamp::connect("192.168.1.3:55443")?;
        lamp.send_cmd(&cmd)?;
    }
    sleep(sleep_dur);
    {
        let mut lamp = block_on(async { AsyncLamp::connect("192.168.1.3:55443").await })?;
        block_on(async { lamp.send_cmd(&cmd2).await })?;
    }
    sleep(sleep_dur);

    Ok(())
}
