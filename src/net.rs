use crate::debugdump;
use crate::debugf;
use crate::errorf;
use crate::infof;
use crate::platform::linux::intr::IrqEntryList;
use crate::util;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;

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
    pub head: Option<Arc<Mutex<NetDevice>>>,
    pub irq_entry_list: IrqEntryList,
}

impl NetDeviceList {
    pub fn net_init() -> Result<NetDeviceList, Box<dyn Error>> {
        infof!("net_init", "initialized")?;

        let irq_entry_list = IrqEntryList::intr_init()?;

        Ok(NetDeviceList {
            head: None,
            irq_entry_list,
        })
    }

    pub fn net_device_register(&mut self, dev: Arc<Mutex<NetDevice>>) -> Result<(), Box<dyn Error>> {
        let mut d = dev.lock().unwrap();
        d.next = self.head.take();
        self.head = Some(dev.clone());

        infof!(
            "net_device_register",
            "registered, dev={}, type={}",
            d.name,
            d.device_type,
        )?;

        Ok(())
    }

    pub fn net_run(&mut self) -> Result<(), Box<dyn Error>> {
        debugf!("net_run", "open all devices...")?;

        self.irq_entry_list.intr_run()?;

        let mut current = self.head.clone();
        while let Some(node) = current {
            let mut n = node.lock().unwrap();
            n.net_device_open()?;
            current = n.next.clone();
        }

        debugf!("net_run", "running...")?;
        Ok(())
    }

    pub fn net_shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        debugf!("net_shutdown", "close all devices...")?;

        let mut current = self.head.clone();
        while let Some(node) = current {
            let mut n = node.lock().unwrap();
            n.net_device_close()?;
            current = n.next.clone();
        }

        debugf!("net_shutdown", "shutting down")?;
        Ok(())
    }

    pub fn net_device_output(&self, name: &str, dev_type: u16, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut current = self.head.clone();
        while let Some(node) = current {
            let n = node.lock().unwrap();
            if n.name == name {
                n.net_device_output(dev_type, data)?;
                break;
            }
            current = n.next.clone();
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct NetDevice {
    pub next: Option<Arc<Mutex<NetDevice>>>,
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
    pub transmit: Option<fn(&NetDevice, u16, &[u8]) -> Result<(), Box<dyn Error>>>,
}

impl NetDevice {
    pub fn net_device_open(&mut self) -> Result<(), Box<dyn Error>> {
        if NET_DEVICE_IS_UP!(self) {
            errorf!("net_device_open", "already opened, dev={}", self.name)?;
            return Err("net device already opened".into());
        }

        if let Some(open) = self.open {
            if let Err(msg) = open() {
                errorf!("net_device_open", "failure, dev={}", self.name)?;
                return Err(msg.into());
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

    pub fn net_device_close(&mut self) -> Result<(), Box<dyn Error>> {
        if !NET_DEVICE_IS_UP!(self) {
            errorf!("net_device_close", "not opened, dev={}", self.name)?;
            return Err("net device not opend".into());
        }

        if let Some(close) = self.close {
            if let Err(msg) = close() {
                errorf!("net_device_close", "failure, dev={}", self.name)?;
                return Err(msg.into());
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

    pub fn net_device_output(&self, dev_type: u16, data: &[u8]) -> Result<(), Box<dyn Error>> {
        if !NET_DEVICE_IS_UP!(self) {
            errorf!("net_device_output", "not opened, dev={}", self.name)?;
            return Err("net device not opened".into());
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
            return Err("net device output too long".into());
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
