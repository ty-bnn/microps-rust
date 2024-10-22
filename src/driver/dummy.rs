use crate::net::{NetDeviceData, NetDeviceOps, NET_DEVICE_TYPE_DUMMY};
use crate::util;
use std::cell::RefCell;
use std::rc::Rc;

const DUMMY_MTU: u16 = u16::MAX;

pub struct DummyDevice {
    data: NetDeviceData,
}

impl NetDeviceOps for DummyDevice {
    fn open(&self) -> Result<(), String> {
        Ok(())
    }
    fn close(&self) -> Result<(), String> {
        Ok(())
    }
    fn transmit(&self, dev_type: u16, data: &[u8]) -> Result<(), String> {
        debugf!(
            "dummy_transmit",
            "dev={}, type=0x{:04x}, len={}",
            self.data.name,
            dev_type,
            data.len()
        )
        .map_err(|e| e.to_string())?;
        debugdump!(data).map_err(|e| e.to_string())?;
        Ok(())
    }
    fn get_data(&self) -> &NetDeviceData {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut NetDeviceData {
        &mut self.data
    }
}

pub fn dummy_init() -> Rc<RefCell<DummyDevice>> {
    Rc::new(RefCell::new(DummyDevice {
        data: NetDeviceData {
            name: String::from("dummy_device"),
            device_type: NET_DEVICE_TYPE_DUMMY,
            mtu: DUMMY_MTU,
            hlen: 0,
            alen: 0,
            ..Default::default()
        },
    }))
}
