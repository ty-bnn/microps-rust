use std::thread::sleep;
use std::time::Duration;

use microps_rust::driver::dummy::dummy_init;
use microps_rust::net::{NetDevices, NetDeviceOps};
use microps_rust::util;
use microps_rust::test::TEST_DATA;

#[macro_use]
extern crate microps_rust;

fn main() -> Result<(), String> {
    let mut devices = NetDevices::new();
    devices.net_init()?;

    let dev = dummy_init();
    devices.net_device_register(dev.clone())?;

    if let Err(msg) = devices.net_run() {
        errorf!("main", "net_run() failure")?;
        return Err(msg)
    }

    loop {
        if let Err(_) = dev.borrow().net_device_output(0x0800, &TEST_DATA) {
            errorf!("main", "net_device_output() failure")?;
            break;
        }
        sleep(Duration::new(1, 0));
    }

    Ok(())
}
