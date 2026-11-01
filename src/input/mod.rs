pub mod uinput;
pub mod xtest;

use anyhow::Result;

pub trait InputBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn key(&mut self, code: u16, down: bool) -> Result<()>;
    fn button(&mut self, button: u16, down: bool) -> Result<()>;
    fn motion(&mut self, absolute: bool, x: i32, y: i32) -> Result<()>;
    fn wheel(&mut self, horizontal: bool, delta: i32) -> Result<()>;
}

pub fn select_backend(preferred: &str) -> Result<Box<dyn InputBackend>> {
    match preferred {
        "xtest" => xtest::XTestBackend::new(),
        "uinput" => uinput::UInputBackend::new(),
        _ => {
            // Auto detect: prefer uinput (works on Wayland and X11), fallback to xtest
            if let Ok(b) = uinput::UInputBackend::new() {
                Ok(b)
            } else {
                xtest::XTestBackend::new()
            }
        }
    }
}
