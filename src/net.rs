use crate::debugdump;
use crate::debugf;
use crate::errorf;
use crate::infof;
use crate::util;

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

pub struct NetDeviceList {
    pub head: Option<Box<NetDevice>>,
}

impl NetDeviceList {
    pub fn new() -> Self {
        NetDeviceList { head: None }
    }

    pub fn net_init(&self) -> Result<(), String> {
        infof!("net_init", "initialized")
    }

    pub fn net_device_register(&mut self, mut dev: Box<NetDevice>) -> Result<(), String> {
        let name = dev.name.clone();
        let device_type = dev.device_type;

        dev.next = self.head.take();
        self.head = Some(dev);

        infof!(
            "net_device_register",
            "registered, dev={}, type={}",
            name,
            device_type
        )?;

        Ok(())
    }

    pub fn net_run(&mut self) -> Result<(), String> {
        debugf!("net_run", "open all devices...")?;

        let mut current = self.head.as_mut();
        while let Some(node) = current {
            node.net_device_open()?;
            current = node.next.as_mut();
        }

        debugf!("net_run", "running...")?;
        Ok(())
    }

    pub fn net_shutdown(&mut self) -> Result<(), String> {
        debugf!("net_shutdown", "close all devices...")?;

        let mut current = self.head.as_mut();
        while let Some(node) = current {
            node.net_device_close()?;
            current = node.next.as_mut();
        }

        debugf!("net_shutdown", "shutting down")?;
        Ok(())
    }

    pub fn net_device_output(&self, name: &str, dev_type: u16, data: &[u8]) -> Result<(), String> {
        let mut current = self.head.as_ref();
        while let Some(dev) = current {
            if dev.name == name {
                dev.net_device_output(dev_type, data)?;
                break;
            }
            current = dev.next.as_ref();
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct NetDevice {
    pub next: Option<Box<NetDevice>>,
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
    pub open: Option<fn() -> Result<(), String>>,
    pub close: Option<fn() -> Result<(), String>>,
    pub transmit: Option<fn(&NetDevice, u16, &[u8]) -> Result<(), String>>,
}

impl NetDevice {
    pub fn net_device_open(&mut self) -> Result<(), String> {
        if NET_DEVICE_IS_UP!(self) {
            errorf!("net_device_open", "already opened, dev={}", self.name)?;
            return Err(String::new());
        }

        if let Some(open) = self.open {
            if let Err(msg) = open() {
                errorf!("net_device_open", "failure, dev={}", self.name)?;
                return Err(msg);
            }
        }

        self.flags |= NET_DEVICE_FLAG_UP;
        infof!(
            "net_device_open",
            "dev={}, state={}",
            self.name,
            NET_DEVICE_STATE!(self)
        )?;

        Ok(())
    }

    pub fn net_device_close(&mut self) -> Result<(), String> {
        if !NET_DEVICE_IS_UP!(self) {
            errorf!("net_device_close", "not opened, dev={}", self.name)?;
            return Err(String::new());
        }

        if let Some(close) = self.close {
            if let Err(msg) = close() {
                errorf!("net_device_close", "failure, dev={}", self.name)?;
                return Err(msg);
            }
        }

        self.flags &= !NET_DEVICE_FLAG_UP;
        infof!(
            "net_device_close",
            "dev={}, state={}",
            self.name,
            NET_DEVICE_STATE!(self)
        )?;

        Ok(())
    }

    pub fn net_device_output(&self, dev_type: u16, data: &[u8]) -> Result<(), String> {
        if !NET_DEVICE_IS_UP!(self) {
            errorf!("net_device_output", "not opened, dev={}", self.name)?;
            return Err(String::new());
        }

        // safe cast.
        if data.len() > u16::MAX as usize || data.len() as u16 > self.mtu {
            errorf!(
                "net_device_output",
                "too long, dev={}, mtu={}, len={}",
                self.name,
                self.mtu,
                data.len()
            )?;
            return Err(String::new());
        }

        debugf!(
            "net_device_output",
            "dev={}, type={}, len={}",
            self.name,
            dev_type,
            data.len()
        )?;
        debugdump!(data)?;

        if let Some(transmit) = self.transmit {
            if let Err(msg) = transmit(self, dev_type, data) {
                errorf!(
                    "net_device_output",
                    "device transmit failure, dev={}, len={}",
                    self.name,
                    data.len()
                )?;
                return Err(msg);
            }
        }

        Ok(())
    }
}
