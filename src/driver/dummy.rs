use crate::net::{NetDevice, NET_DEVICE_TYPE_DUMMY};
use crate::util;

const DUMMY_MTU: u16 = u16::MAX;
pub const DUMMY_DEV_NAME: &str = "dummy";

fn transmit(dev: &NetDevice, dev_type: u16, data: &[u8]) -> Result<(), String> {
    debugf!(
        "dummy_transmit",
        "dev={}, type=0x{:04x}, len={}",
        dev.name,
        dev_type,
        data.len()
    )
    .map_err(|e| e.to_string())?;
    debugdump!(data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn dummy_init() -> Box<NetDevice> {
    Box::new(NetDevice {
        name: String::from(DUMMY_DEV_NAME),
        device_type: NET_DEVICE_TYPE_DUMMY,
        mtu: DUMMY_MTU,
        hlen: 0,
        alen: 0,
        transmit: Some(transmit),
        ..Default::default()
    })
}
