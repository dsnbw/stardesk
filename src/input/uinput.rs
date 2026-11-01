use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;

use super::InputBackend;

pub struct UInputBackend {
    file: Option<File>,
    width: i32,
    height: i32,
}

impl UInputBackend {
    pub fn new() -> Result<Box<dyn InputBackend>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open("/dev/uinput")
            .or_else(|_| {
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open("/dev/input/uinput")
            })
            .context("opening /dev/uinput. Ensure 'uinput' kernel module is loaded and permissions are configured.")?;

        let mut backend = Self {
            file: Some(file),
            width: 1920,
            height: 1080,
        };
        backend.init_device()?;
        Ok(Box::new(backend))
    }

    fn init_device(&mut self) -> Result<()> {
        let fd = self.file.as_ref().unwrap().as_raw_fd();
        unsafe {
            // Enable key events
            libc::ioctl(fd, UI_SET_EVBIT, libc::EV_KEY);
            libc::ioctl(fd, UI_SET_EVBIT, libc::EV_SYN);
            libc::ioctl(fd, UI_SET_EVBIT, libc::EV_REL);
            libc::ioctl(fd, UI_SET_EVBIT, libc::EV_ABS);

            // Enable keys 0..255
            for key in 1..255 {
                libc::ioctl(fd, UI_SET_KEYBIT, key);
            }
            // Mouse buttons
            libc::ioctl(fd, UI_SET_KEYBIT, BTN_LEFT);
            libc::ioctl(fd, UI_SET_KEYBIT, BTN_RIGHT);
            libc::ioctl(fd, UI_SET_KEYBIT, BTN_MIDDLE);
            libc::ioctl(fd, UI_SET_KEYBIT, BTN_SIDE);
            libc::ioctl(fd, UI_SET_KEYBIT, BTN_EXTRA);

            // Rel motion and wheel
            libc::ioctl(fd, UI_SET_RELBIT, libc::REL_X);
            libc::ioctl(fd, UI_SET_RELBIT, libc::REL_Y);
            libc::ioctl(fd, UI_SET_RELBIT, libc::REL_WHEEL);
            libc::ioctl(fd, UI_SET_RELBIT, libc::REL_HWHEEL);

            // Abs motion
            libc::ioctl(fd, UI_SET_ABSBIT, libc::ABS_X);
            libc::ioctl(fd, UI_SET_ABSBIT, libc::ABS_Y);

            // Create virtual device
            let mut uidev: uinput_user_dev = std::mem::zeroed();
            let name = b"StarDesk Virtual Input\0";
            uidev.name[..name.len()].copy_from_slice(name);
            uidev.id.bustype = libc::BUS_USB;
            uidev.id.vendor = 0x1532;
            uidev.id.product = 0x0001;
            uidev.id.version = 1;

            uidev.absmin[libc::ABS_X as usize] = 0;
            uidev.absmax[libc::ABS_X as usize] = 65535;
            uidev.absmin[libc::ABS_Y as usize] = 0;
            uidev.absmax[libc::ABS_Y as usize] = 65535;

            let dev_slice = std::slice::from_raw_parts(
                &uidev as *const _ as *const u8,
                std::mem::size_of::<uinput_user_dev>(),
            );
            if libc::write(fd, dev_slice.as_ptr() as *const _, dev_slice.len()) < 0 {
                anyhow::bail!("failed to write uinput device description");
            }

            if libc::ioctl(fd, UI_DEV_CREATE) < 0 {
                anyhow::bail!("failed to issue UI_DEV_CREATE");
            }
        }
        Ok(())
    }

    fn emit(&mut self, type_: u16, code: u16, val: i32) -> Result<()> {
        let fd = self.file.as_ref().unwrap().as_raw_fd();
        let ev = libc::input_event {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_,
            code,
            value: val,
        };
        unsafe {
            libc::write(
                fd,
                &ev as *const _ as *const _,
                std::mem::size_of::<libc::input_event>(),
            );
        }
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        self.emit(libc::EV_SYN as u16, libc::SYN_REPORT as u16, 0)
    }
}

impl Drop for UInputBackend {
    fn drop(&mut self) {
        if let Some(f) = self.file.take() {
            unsafe {
                libc::ioctl(f.as_raw_fd(), UI_DEV_DESTROY);
            }
        }
    }
}

impl InputBackend for UInputBackend {
    fn name(&self) -> &'static str {
        "uinput"
    }

    fn key(&mut self, vk_code: u16, down: bool) -> Result<()> {
        let linux_key = win_vk_to_linux(vk_code);
        self.emit(libc::EV_KEY as u16, linux_key, if down { 1 } else { 0 })?;
        self.sync()
    }

    fn button(&mut self, btn: u16, down: bool) -> Result<()> {
        let code = match btn {
            1 => BTN_LEFT,
            2 => BTN_RIGHT,
            3 => BTN_MIDDLE,
            4 => BTN_SIDE,
            5 => BTN_EXTRA,
            _ => BTN_LEFT,
        };
        self.emit(libc::EV_KEY as u16, code, if down { 1 } else { 0 })?;
        self.sync()
    }

    fn motion(&mut self, absolute: bool, x: i32, y: i32) -> Result<()> {
        if absolute {
            self.emit(libc::EV_ABS as u16, libc::ABS_X as u16, x)?;
            self.emit(libc::EV_ABS as u16, libc::ABS_Y as u16, y)?;
        } else {
            self.emit(libc::EV_REL as u16, libc::REL_X as u16, x)?;
            self.emit(libc::EV_REL as u16, libc::REL_Y as u16, y)?;
        }
        self.sync()
    }

    fn wheel(&mut self, horizontal: bool, delta: i32) -> Result<()> {
        let axis = if horizontal {
            libc::REL_HWHEEL as u16
        } else {
            libc::REL_WHEEL as u16
        };
        self.emit(libc::EV_REL as u16, axis, delta)?;
        self.sync()
    }
}

// UInput Constants
const UI_SET_EVBIT: libc::c_ulong = 0x40045564;
const UI_SET_KEYBIT: libc::c_ulong = 0x40045565;
const UI_SET_RELBIT: libc::c_ulong = 0x40045566;
const UI_SET_ABSBIT: libc::c_ulong = 0x40045567;
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
const UI_DEV_DESTROY: libc::c_ulong = 0x5502;

const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;
const BTN_SIDE: u16 = 0x113;
const BTN_EXTRA: u16 = 0x114;

#[repr(C)]
struct uinput_user_dev {
    name: [u8; 80],
    id: input_id,
    ff_effects_max: u32,
    absmax: [i32; 64],
    absmin: [i32; 64],
    absfuzz: [i32; 64],
    absflat: [i32; 64],
}

#[repr(C)]
struct input_id {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

// Map Windows Virtual Keycodes to Linux Keycodes
fn win_vk_to_linux(vk: u16) -> u16 {
    match vk {
        0x08 => 14,  // Backspace
        0x09 => 15,  // Tab
        0x0D => 28,  // Enter
        0x10 => 42,  // Left Shift
        0x11 => 29,  // Left Ctrl
        0x12 => 56,  // Left Alt
        0x14 => 58,  // CapsLock
        0x1B => 1,   // Escape
        0x20 => 57,  // Space
        0x25 => 105, // Left
        0x26 => 103, // Up
        0x27 => 106, // Right
        0x28 => 108, // Down
        0x2E => 111, // Delete
        // A..Z -> KEY_A..KEY_Z
        0x41..=0x5A => {
            let offset = vk - 0x41;
            const LETTERS: [u16; 26] = [
                30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22,
                47, 17, 45, 21, 44,
            ];
            LETTERS[offset as usize]
        }
        // 0..9 -> KEY_0..KEY_9
        0x30 => 11,
        0x31..=0x39 => vk - 0x31 + 2,
        _ => 0,
    }
}
