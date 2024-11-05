use crate::errorf;
use crate::util;

use crate::net::NetDevice;

use libc::{raise, SIGHUP, SIGRTMIN};
use signal_hook::iterator::backend::Handle;
use signal_hook::iterator::Signals;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const INTR_IRQ_SHARED: u16 = 0x0001;

pub struct IrqEntryList {
    head: Option<Arc<Mutex<IrqEntry>>>,
    signals: Vec<i32>,
    signals_handle: Option<Handle>,
}

impl IrqEntryList {
    pub fn intr_init() -> Result<IrqEntryList, Box<dyn Error>> {
        debugf!("intr_init", "initialized")?;

        Ok(IrqEntryList {
            head: None,
            signals: vec![SIGHUP, SIGRTMIN() + 1],
            signals_handle: None,
        })
    }

    pub fn intr_run(&mut self) -> Result<(), Box<dyn Error>> {
        debugf!("intr_run", "signal thread running...")?;

        let signals = Signals::new(&self.signals)?;
        self.signals_handle = Some(signals.handle());
        if let Some(irq_head) = self.head.clone() {
            thread::spawn(move || -> Result<(), String> {
                Self::intr_thread(signals, irq_head)?;
                Ok(())
            });
        }

        thread::sleep(Duration::from_secs(3));
        Ok(())
    }

    fn intr_thread(mut signals: Signals, irq_head: Arc<Mutex<IrqEntry>>) -> Result<(), String> {
        debugf!("intr_thread", "signal handler running...")?;
        for sig in signals.forever() {
            match sig {
                SIGHUP => {
                    break;
                }
                _ => {
                    let mut current = Some(irq_head.clone());
                    while let Some(entry) = current {
                        let e = entry.lock().unwrap();
                        if e.irq == sig {
                            debugf!("intr_thread", "irq={}, name={}", e.irq, e.name)?;
                            if let Some(handler) = e.handler {
                                handler(e.irq, e.dev.clone())?;
                                break;
                            }
                        }
                        current = e.next.clone();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn intr_request_irq(
        &mut self,
        irq: i32,
        handler: Option<fn(i32, Arc<Mutex<NetDevice>>) -> Result<(), String>>,
        flags: u16,
        name: &str,
        dev: Arc<Mutex<NetDevice>>,
    ) -> Result<(), Box<dyn Error>> {
        // check if the irq is already registered.
        let mut current = self.head.clone();
        while let Some(entry) = current {
            let e = entry.lock().unwrap();
            if e.irq == irq {
                if (e.flags ^ INTR_IRQ_SHARED) != 0 || (flags ^ INTR_IRQ_SHARED) != 0 {
                    errorf!("intr_request_irq", "conflicts with already registered IRQs")?;
                    return Err("error".into());
                }
            }
            current = e.next.clone();
        }

        let entry = Some(Arc::new(Mutex::new(IrqEntry {
            next: self.head.take(),
            irq,
            handler,
            flags,
            name: name.to_string(),
            dev,
        })));

        self.head = entry;

        // if let Some(handle) = self.signals_handle.as_ref() {
        //     handle.close();
        // }
        // self.signals.push(irq);
        // self.intr_run()?; // restart signal thread.

        Ok(())
    }
}

pub fn intr_raise_irq(irq: i32) {
    unsafe {
        raise(irq);
    }
}

pub struct IrqEntry {
    next: Option<Arc<Mutex<IrqEntry>>>,
    irq: i32,
    handler: Option<fn(i32, Arc<Mutex<NetDevice>>) -> Result<(), String>>,
    flags: u16,
    name: String,
    dev: Arc<Mutex<NetDevice>>,
}
