use crate::debugdump;
use crate::debugf;
use crate::errorf;
use crate::infof;
use crate::util;
use std::cell::RefCell;
use std::rc::Rc;

const NET_DEVICE_ADDR_LEN: usize = 16;
const NET_DEVICE_FLAG_UP: u16 = 0x0001;

pub const NET_DEVICE_TYPE_DUMMY: u16 = 0x0000;

macro_rules! NET_DEVICE_IS_UP {
    ($dev:expr) => {
        ($dev.flags & NET_DEVICE_FLAG_UP) != 0
    };
}

macro_rules! NET_DEVICE_STATE {
    ($dev:expr) => {
        if NET_DEVICE_IS_UP!($dev) {
            "up"
        } else {
            "down"
        }
    };
}

pub struct NetDevices<T: NetDeviceOps> {
    pub devices: Vec<Rc<RefCell<T>>>,
}

impl<T> NetDevices<T>
where
    T: NetDeviceOps,
{
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn net_init(&self) -> Result<(), String> {
        infof!("net_init", "initialized")
    }

    pub fn net_device_register(&mut self, dev: Rc<RefCell<T>>) -> Result<(), String> {
        self.devices.push(dev.clone());
        infof!(
            "net_device_register",
            "registered, dev={}, type={}",
            dev.borrow().get_data().name,
            dev.borrow().get_data().device_type
        )?;
        Ok(())
    }

    pub fn net_run(&self) -> Result<(), String> {
        debugf!("net_run", "open all devices...")?;

        for dev in &self.devices {
            dev.borrow_mut().net_device_open()?;
        }

        debugf!("net_run", "running...")?;
        Ok(())
    }

    pub fn net_shutdown(&self) -> Result<(), String> {
        debugf!("net_shutdown", "close all devices...")?;

        for dev in &self.devices {
            dev.borrow_mut().net_device_close()?;
        }

        debugf!("net_shutdown", "shutting down")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct NetDeviceData {
    pub index: u32,
    pub name: String,
    pub device_type: u16,
    pub mtu: u16,
    pub flags: u16,
    pub hlen: u16, /* header length */
    pub alen: u16, /* address length */
    pub addr: [u8; NET_DEVICE_ADDR_LEN],
    pub peer: [u8; NET_DEVICE_ADDR_LEN],
    pub broadcast: [u8; NET_DEVICE_ADDR_LEN],
}

pub trait NetDeviceOps {
    fn open(&self) -> Result<(), String>;

    fn close(&self) -> Result<(), String>;

    fn transmit(&self, dev_type: u16, data: &[u8]) -> Result<(), String>;

    fn get_data(&self) -> &NetDeviceData; /* this method is used to read NetDeviceData in Device */

    fn get_data_mut(&mut self) -> &mut NetDeviceData; /* this method is used to change NetDeviceData in Device */

    fn net_device_open(&mut self) -> Result<(), String> {
        let data = self.get_data();
        if NET_DEVICE_IS_UP!(data) {
            errorf!("net_device_open", "already opened, dev={}", data.name)?;
            return Err(String::new());
        }

        if let Err(msg) = self.open() {
            errorf!("net_device_open", "failure, dev={}", data.name)?;
            return Err(msg);
        }

        let data = self.get_data_mut();
        data.flags |= NET_DEVICE_FLAG_UP;
        infof!(
            "net_device_open",
            "dev={}, state={}",
            data.name,
            NET_DEVICE_STATE!(data)
        )?;

        Ok(())
    }

    fn net_device_close(&mut self) -> Result<(), String> {
        let data = self.get_data();
        if !NET_DEVICE_IS_UP!(data) {
            errorf!("net_device_close", "not opened, dev={}", data.name)?;
            return Err(String::new());
        }

        if let Err(msg) = self.close() {
            errorf!("net_device_close", "failure, dev={}", data.name)?;
            return Err(msg);
        }

        let data = self.get_data_mut();
        data.flags &= !NET_DEVICE_FLAG_UP;
        infof!(
            "net_device_close",
            "dev={}, state={}",
            data.name,
            NET_DEVICE_STATE!(data)
        )?;

        Ok(())
    }

    fn net_device_output(&self, dev_type: u16, data: &[u8]) -> Result<(), String> {
        let dev_data = self.get_data();
        if !NET_DEVICE_IS_UP!(dev_data) {
            errorf!("net_device_output", "not opened, dev={}", dev_data.name)?;
            return Err(String::new());
        }

        // safe cast.
        if data.len() > u16::MAX as usize || data.len() as u16 > dev_data.mtu {
            errorf!(
                "net_device_output",
                "too long, dev={}, mtu={}, len={}",
                dev_data.name,
                dev_data.mtu,
                data.len()
            )?;
            return Err(String::new());
        }

        debugf!(
            "net_device_output",
            "dev={}, type={}, len={}",
            dev_data.name,
            dev_type,
            data.len()
        )?;
        debugdump!(data)?;

        if let Err(msg) = self.transmit(dev_type, data) {
            errorf!(
                "net_device_output",
                "device transmit failure, dev={}, len={}",
                dev_data.name,
                data.len()
            )?;
            return Err(msg);
        }

        Ok(())
    }
}
