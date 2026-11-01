use anyhow::Result;
use std::process::Command;

use super::InputBackend;

pub struct XTestBackend;

impl XTestBackend {
    pub fn new() -> Result<Box<dyn InputBackend>> {
        Ok(Box::new(Self))
    }
}

impl InputBackend for XTestBackend {
    fn name(&self) -> &'static str {
        "xtest"
    }

    fn key(&mut self, vk_code: u16, down: bool) -> Result<()> {
        let action = if down { "keydown" } else { "keyup" };
        let _ = Command::new("xdotool")
            .args([action, &vk_code.to_string()])
            .status();
        Ok(())
    }

    fn button(&mut self, btn: u16, down: bool) -> Result<()> {
        let action = if down { "mousedown" } else { "mouseup" };
        let _ = Command::new("xdotool")
            .args([action, &btn.to_string()])
            .status();
        Ok(())
    }

    fn motion(&mut self, absolute: bool, x: i32, y: i32) -> Result<()> {
        if absolute {
            let _ = Command::new("xdotool")
                .args(["mousemove", &x.to_string(), &y.to_string()])
                .status();
        } else {
            let _ = Command::new("xdotool")
                .args(["mousemove_relative", "--", &x.to_string(), &y.to_string()])
                .status();
        }
        Ok(())
    }

    fn wheel(&mut self, horizontal: bool, delta: i32) -> Result<()> {
        let button = if horizontal {
            if delta > 0 { "7" } else { "6" }
        } else {
            if delta > 0 { "4" } else { "5" }
        };
        let _ = Command::new("xdotool")
            .args(["click", button])
            .status();
        Ok(())
    }
}
