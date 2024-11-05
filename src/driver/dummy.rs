use crate::net::{NetDevice, NET_DEVICE_TYPE_DUMMY};
use crate::platform::linux::intr::{intr_raise_irq, IrqEntryList, INTR_IRQ_SHARED};
use crate::util;
use libc::SIGRTMIN;
use std::error::Error;
use std::sync::{Arc, Mutex};

const DUMMY_MTU: u16 = u16::MAX;
pub const DUMMY_DEV_NAME: &str = "dummy";

fn dummy_transimt(dev: &NetDevice, dev_type: u16, data: &[u8]) -> Result<(), String> {
    debugf!(
        "dummy_transmit",
        "dev={}, type=0x{:04x}, len={}",
        dev.name,
        dev_type,
        data.len()
    )
    .map_err(|e| e.to_string())?;
    debugdump!(data).map_err(|e| e.to_string())?;

    intr_raise_irq(SIGRTMIN() + 1);

    Ok(())
}

fn dummy_isr(irq: i32, dev: Arc<Mutex<NetDevice>>) -> Result<(), String> {
    debugf!("dummy_isr", "irq={}, dev={}", irq, dev.lock().unwrap().name)?;
    Ok(())
}

pub fn dummy_init(irq_entries: &mut IrqEntryList) -> Result<Arc<Mutex<NetDevice>>, Box<dyn Error>> {
    let dummy_dev = Arc::new(Mutex::new(NetDevice {
        name: String::from(DUMMY_DEV_NAME),
        device_type: NET_DEVICE_TYPE_DUMMY,
        mtu: DUMMY_MTU,
        hlen: 0,
        alen: 0,
        transmit: Some(dummy_transimt),
        ..Default::default()
    }));

    irq_entries.intr_request_irq(
        SIGRTMIN() + 1,
        Some(dummy_isr),
        INTR_IRQ_SHARED,
        DUMMY_DEV_NAME,
        dummy_dev.clone(),
    )?;

    Ok(dummy_dev)
}
