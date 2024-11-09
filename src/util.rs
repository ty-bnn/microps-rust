use chrono::Local;
use fs2::FileExt;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Write};

pub fn lprintf(
    level: char,
    file: &str,
    line: u32,
    func: &str,
    args: fmt::Arguments,
) -> Result<(), Box<dyn Error>> {
    writeln!(
        &mut io::stderr().lock(),
        "{} [{}] {}: {} ({}, {})",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        level,
        func,
        args,
        file,
        line
    )?;
    Ok(())
}

pub fn hexdump(data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut stderr = File::create("/dev/stderr")?;
    stderr.lock_exclusive()?;

    writeln!(
        stderr,
        "+------+-------------------------------------------------+------------------+"
    )?;
    for offset in (0..data.len()).step_by(16) {
        write!(stderr, "| {:04x} | ", offset)?;
        for index in 0..16 {
            if offset + index < data.len() {
                write!(stderr, "{:02x} ", 0xff & data[offset + index])?;
            } else {
                write!(stderr, "   ")?;
            }
        }
        write!(stderr, "| ")?;
        for index in 0..16 {
            if offset + index < data.len() {
                if data[offset + index].is_ascii() && data[offset + index].is_ascii_graphic() {
                    write!(stderr, "{}", data[offset + index] as char)?;
                } else {
                    write!(stderr, ".")?;
                }
            } else {
                write!(stderr, " ")?;
            }
        }
        writeln!(stderr, " |")?;
    }
    writeln!(
        stderr,
        "+------+-------------------------------------------------+------------------+"
    )?;

    stderr.unlock()?;
    Ok(())
}

#[macro_export]
macro_rules! errorf {
    ($function_name:expr, $($arg:tt)*) => {{
        util::lprintf('E', file!(), line!(), $function_name, format_args!($($arg)*)).map_err(|e| e.to_string())
    }};
}

#[macro_export]
macro_rules! warnf {
    ($function_name:expr, $($arg:tt)*) => {{
        util::lprintf('W', file!(), line!(), $function_name, format_args!($($arg)*)).map_err(|e| e.to_string())
    }};
}

#[macro_export]
macro_rules! infof {
    ($function_name:expr, $($arg:tt)*) => {{
        util::lprintf('I', file!(), line!(), $function_name, format_args!($($arg)*)).map_err(|e| e.to_string())
    }};
}

#[macro_export]
macro_rules! debugf {
    ($function_name:expr, $($arg:tt)*) => {{
        util::lprintf('D', file!(), line!(), $function_name, format_args!($($arg)*)).map_err(|e| e.to_string())
    }};
}

#[macro_export]
macro_rules! debugdump {
    ($data:expr) => {
        util::hexdump($data).map_err(|e| e.to_string())
    };
}
