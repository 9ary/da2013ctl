use std::os::unix::io;

use byteorder::{LittleEndian, WriteBytesExt};
use nix;
use nix::fcntl;
use nix::sys::stat::Mode;
use nix::unistd;

mod hid_ioctl {
    const HID_IOC_MAGIC: u8 = b'H';
    const HID_IOC_NR_SFEATURE: u8 = 0x06;
    const HID_IOC_NR_GFEATURE: u8 = 0x07;

    ioctl!(readwrite buf sfeature with HID_IOC_MAGIC, HID_IOC_NR_SFEATURE; u8);
    ioctl!(readwrite buf gfeature with HID_IOC_MAGIC, HID_IOC_NR_GFEATURE; u8);
}

const DA2013_CMD_SET: u16 = 0x0300;
const DA2013_REQ_DPI: u16 = 0x0104;
const DA2013_REQ_LED: u16 = 0x0003;
const DA2013_REQ_FREQ: u16 = 0x0500;

pub struct Da2013 {
    fd: io::RawFd,
}

pub enum Freq {
    F125,
    F500,
    F1000,
}

pub enum Led {
    Logo,
    Wheel,
}

impl Da2013 {
    pub fn open<P: nix::NixPath>(path: P) -> Result<Da2013, nix::Error> {
        let fd = fcntl::open(&path, fcntl::O_RDWR, Mode::empty())?;
        Ok(Da2013 {
            fd: fd,
        })
    }

    fn do_cmd(&self, command: u16, request: u16, arg0: u16, arg1: u16, footer: u8) {
        let mut buf = Vec::with_capacity(91);

        // HID report number
        buf.write_u8(0).unwrap();
        // Status
        buf.write_u8(0).unwrap();
        // Padding
        for _ in 0..3 { buf.write_u8(0).unwrap(); }
        buf.write_u16::<LittleEndian>(command).unwrap();
        buf.write_u16::<LittleEndian>(request).unwrap();
        buf.write_u16::<LittleEndian>(arg0).unwrap();
        buf.write_u16::<LittleEndian>(arg1).unwrap();
        // Padding
        for _ in 0..76 { buf.write_u8(0).unwrap(); }
        buf.write_u8(footer).unwrap();
        // Padding
        buf.write_u8(0).unwrap();

        // Errors here are considered non-critical
        // We try multiple times to make sure the device has registered the command, sometimes it
        // doesn't
        for _ in 0..3 {
            unsafe {
                match hid_ioctl::sfeature(self.fd, buf.as_mut_ptr(), buf.len()) {
                    Ok(_) => match hid_ioctl::gfeature(self.fd, buf.as_mut_ptr(), buf.len()) {
                        // Status field of the response
                        Ok(_) => match buf[1] {
                            // We expect the same values as librazer/razercfg does
                            0 ... 3 => {},
                            e => println!("Command {:#X}/{:#X} failed with {:#X}",
                                          command, request, e),
                        },
                        Err(e) => println!("HIDIOCGFEATURE: {}", e.to_string()),
                    },
                    Err(e) => println!("HIDIOCGFEATURE: {}", e.to_string()),
                }
            }
        }
    }

    pub fn set_res(&self, res: i32) {
        let res = (res / 100 - 1) * 4;
        let arg0 = (res | (res << 8)) as u16;
        self.do_cmd(DA2013_CMD_SET, DA2013_REQ_DPI, arg0, 0, 0x06);
    }

    pub fn set_freq(&self, freq: Freq) {
        let (arg0, footer) = match freq {
            Freq::F125 => (8, 0x0C),
            Freq::F500 => (2, 0x06),
            Freq::F1000 => (1, 0x05),
        };
        self.do_cmd(DA2013_CMD_SET, DA2013_REQ_FREQ, arg0, 0, footer);
    }

    pub fn set_led(&self, led: Led, state: bool) {
        let (arg0, arg1, footer) = match (led, state) {
            (Led::Logo, true) => (0x0401, 1, 0x04),
            (Led::Logo, false) => (0x0401, 0, 0x05),
            (Led::Wheel, true) => (0x0101, 1, 0x01),
            (Led::Wheel, false) => (0x0101, 0, 0x00),
        };
        self.do_cmd(DA2013_CMD_SET, DA2013_REQ_LED, arg0, arg1, footer);
    }
}

impl Drop for Da2013 {
    fn drop(&mut self) {
        unistd::close(self.fd).expect("Failed to close device");
    }
}
