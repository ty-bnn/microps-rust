use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use microps_rust::driver::dummy::{dummy_init, DUMMY_DEV_NAME};
use microps_rust::net::NetDeviceList;
use microps_rust::test::TEST_DATA;
use microps_rust::util;
use signal_hook::consts::SIGINT;
use signal_hook::flag;

#[macro_use]
extern crate microps_rust;

fn main() -> Result<(), Box<dyn Error>> {
    let term = Arc::new(AtomicBool::new(false));

    flag::register(SIGINT, Arc::clone(&term))?;

    let mut devices = NetDeviceList::new();
    devices.net_init()?;

    let dev = dummy_init();
    devices.net_device_register(dev)?;

    if let Err(msg) = devices.net_run() {
        errorf!("main", "net_run() failure")?;
        return Err(msg.into());
    }

    while !term.load(Ordering::Relaxed) {
        if let Err(_) = devices.net_device_output(DUMMY_DEV_NAME, 0x0800, &TEST_DATA) {
            errorf!("main", "net_device_output() failure")?;
            break;
        }
        sleep(Duration::new(1, 0));
    }

    devices.net_shutdown()?;

    Ok(())
}
