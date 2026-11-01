use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

use crate::config::Config;

const STARDESK_FRAME_MAGIC: u32 = 0x53544152;
const STARDESK_FRAME_VERSION: u32 = 1;
const STARDESK_FORMAT_BGRA32: u32 = 1;

#[repr(C, packed)]
pub struct FrameHeader {
    pub magic: u32,
    pub version: u32,
    pub sequence: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: u32,
    pub timestamp_us: u64,
    pub payload_bytes: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub cursor_visible: u8,
    pub reserved: [u8; 15],
}

pub struct SharedFrameBuffer {
    ptr: *mut u8,
    size: usize,
    shm_fd: i32,
}

impl SharedFrameBuffer {
    pub fn create(width: u32, height: u32) -> Result<Self> {
        let stride = width * 4;
        let payload_size = (stride * height) as usize;
        let total_size = std::mem::size_of::<FrameHeader>() + payload_size;

        // Create POSIX shared memory object
        let shm_name = b"/stardesk-framebuffer\0";
        unsafe {
            let fd = libc::shm_open(
                shm_name.as_ptr() as *const _,
                libc::O_CREAT | libc::O_RDWR | libc::O_TRUNC,
                0o666,
            );
            if fd < 0 {
                anyhow::bail!("Failed to shm_open /stardesk-framebuffer");
            }
            if libc::ftruncate(fd, total_size as libc::off_t) < 0 {
                libc::close(fd);
                anyhow::bail!("Failed to ftruncate shm");
            }
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                total_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );
            if ptr == libc::MAP_FAILED {
                libc::close(fd);
                anyhow::bail!("Failed to mmap shm");
            }

            // Zero and init header
            let header = ptr as *mut FrameHeader;
            (*header).magic = STARDESK_FRAME_MAGIC;
            (*header).version = STARDESK_FRAME_VERSION;
            (*header).sequence = 0;
            (*header).width = width;
            (*header).height = height;
            (*header).stride = stride;
            (*header).format = STARDESK_FORMAT_BGRA32;
            (*header).payload_bytes = payload_size as u32;

            Ok(Self {
                ptr: ptr as *mut u8,
                size: total_size,
                shm_fd: fd,
            })
        }
    }
}

impl Drop for SharedFrameBuffer {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut _, self.size);
            libc::close(self.shm_fd);
            libc::shm_unlink(b"/stardesk-framebuffer\0".as_ptr() as *const _);
        }
    }
}

pub fn run() -> Result<()> {
    let config = Config::load()?;
    println!("Starting StarDesk native Linux capture service...");
    println!("Capture backend: {}", config.capture_backend);
    println!("Capture source: {}", config.capture_source);

    let shm = SharedFrameBuffer::create(1920, 1080)
        .context("initializing shared frame buffer for Wine hook")?;

    // In a production Wayland session, PipeWire ScreenCast portal feeds frames.
    // In an X11 session, XShm feeds frames.
    // Loop keeps capture active while session runs.
    loop {
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps tick
    }
}
