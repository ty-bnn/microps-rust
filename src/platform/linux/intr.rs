use crate::errorf;
use crate::util;

use crate::net::NetDevice;

use signal_hook::iterator::SignalsInfo;
use std::error::Error;

const INTR_IRQ_SHARED: u16 = 0x0001;

pub struct IrqEntryList {
    head: Option<Box<IrqEntry>>,
}

impl IrqEntryList {
    pub fn intr_request_irq(
        &mut self,
        irq: i32,
        handler: Option<fn(u32, &NetDevice) -> Result<(), String>>,
        flags: u16,
        name: &str,
        dev: Box<NetDevice>,
        signals: &SignalsInfo,
    ) -> Result<(), Box<dyn Error>> {
        // check irq is already registered.
        let current = self.head.as_ref();
        while let Some(entry) = current {
            if entry.irq == irq {
                if (entry.flags ^ INTR_IRQ_SHARED) != 0 || (flags ^ INTR_IRQ_SHARED) != 0 {
                    errorf!("intr_request_irq", "conflicts with already registered IRQs")?;
                    return Err("error".into());
                }
            }
        }

        let entry = Some(Box::new(IrqEntry {
            next: self.head.take(),
            irq,
            handler,
            flags,
            name: name.to_string(),
            dev,
        }));

        self.head = entry;

        signals.add_signal(irq)?;

        Ok(())
    }
}

pub struct IrqEntry {
    next: Option<Box<IrqEntry>>,
    irq: i32,
    handler: Option<fn(u32, &NetDevice) -> Result<(), String>>,
    flags: u16,
    name: String,
    dev: Box<NetDevice>,
}
