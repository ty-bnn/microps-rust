mod util;

use std::io;

fn main() -> io::Result<()> {
    debugf!("main", "Hello {} !", "takayuki")?;
    let data: [u8; 48] = [
        0x45, 0x00, 0x00, 0x30,
        0x00, 0x80, 0x00, 0x00,
        0xff, 0x01, 0xbd, 0x4a,
        0x7f, 0x00, 0x00, 0x01,
        0x7f, 0x00, 0x00, 0x01,
        0x08, 0x00, 0x35, 0x64,
        0x00, 0x80, 0x00, 0x01,
        0x31, 0x32, 0x33, 0x34,
        0x35, 0x36, 0x37, 0x38,
        0x39, 0x30, 0x21, 0x40,
        0x23, 0x24, 0x25, 0x5e,
        0x26, 0x2a, 0x28, 0x29
    ];

    util::hexdump(&data)?;

    Ok(())
}
