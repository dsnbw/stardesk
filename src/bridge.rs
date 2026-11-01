use anyhow::{Context, Result};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::Config;
use crate::input::{self, InputBackend};

const STARDESK_MAGIC: u32 = 0x53544152;
const STARDESK_RECORD_HELLO: u16 = 1;
const STARDESK_RECORD_MOUSE: u16 = 2;
const STARDESK_RECORD_KEYBOARD: u16 = 3;

pub fn serve() -> Result<()> {
    let config = Config::load()?;
    let backend = input::select_backend(&config.input_backend)?;
    let backend_name = backend.name();
    println!("StarDesk input bridge initialized with backend: {}", backend_name);

    let shared_backend = Arc::new(Mutex::new(backend));
    let listener = TcpListener::bind(("127.0.0.1", config.bridge_port))
        .with_context(|| format!("binding loopback bridge on port {}", config.bridge_port))?;

    // Record runtime port for hook DLL
    let prefix = crate::config::wine_prefix()?;
    let runtime_path = prefix.join("drive_c").join("stardesk-bridge.runtime");
    let _ = std::fs::write(&runtime_path, format!("port={}\n", config.bridge_port));

    println!("Listening for StarDesk Wine hook on 127.0.0.1:{}", config.bridge_port);

    for stream in listener.incoming().flatten() {
        let backend = shared_backend.clone();
        thread::spawn(move || {
            if let Err(e) = handle_connection(stream, backend) {
                eprintln!("Hook connection closed: {:#}", e);
            }
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, backend: Arc<Mutex<Box<dyn InputBackend>>>) -> Result<()> {
    let mut header = [0u8; 6];
    loop {
        if stream.read_exact(&mut header).is_err() {
            break;
        }
        let magic = u32::from_le_bytes(header[0..4].try_into().unwrap());
        let version = u16::from_le_bytes(header[4..6].try_into().unwrap());
        if magic != STARDESK_MAGIC || version != 1 {
            anyhow::bail!("Invalid magic or version from hook");
        }

        let mut kind_buf = [0u8; 2];
        stream.read_exact(&mut kind_buf)?;
        let kind = u16::from_le_bytes(kind_buf);

        let mut backend_guard = backend.lock().unwrap();

        match kind {
            STARDESK_RECORD_HELLO => {
                // Handshake acknowledged
            }
            STARDESK_RECORD_MOUSE => {
                let mut body = [0u8; 12];
                stream.read_exact(&mut body)?;
                let flags = u16::from_le_bytes(body[0..2].try_into().unwrap());
                let dx = i32::from_le_bytes(body[2..6].try_into().unwrap());
                let dy = i32::from_le_bytes(body[6..10].try_into().unwrap());
                let data = i16::from_le_bytes(body[10..12].try_into().unwrap());

                // MOUSEEVENTF_MOVE = 0x0001, ABSOLUTE = 0x8000
                let is_abs = (flags & 0x8000) != 0;
                let is_move = (flags & 0x0001) != 0;
                if is_move {
                    let _ = backend_guard.motion(is_abs, dx, dy);
                }

                // Left down / up
                if (flags & 0x0002) != 0 { let _ = backend_guard.button(1, true); }
                if (flags & 0x0004) != 0 { let _ = backend_guard.button(1, false); }
                // Right down / up
                if (flags & 0x0008) != 0 { let _ = backend_guard.button(2, true); }
                if (flags & 0x0010) != 0 { let _ = backend_guard.button(2, false); }
                // Middle down / up
                if (flags & 0x0020) != 0 { let _ = backend_guard.button(3, true); }
                if (flags & 0x0040) != 0 { let _ = backend_guard.button(3, false); }
                // Wheel
                if (flags & 0x0800) != 0 {
                    let _ = backend_guard.wheel(false, data as i32 / 120);
                }
                // HWheel
                if (flags & 0x1000) != 0 {
                    let _ = backend_guard.wheel(true, data as i32 / 120);
                }
            }
            STARDESK_RECORD_KEYBOARD => {
                let mut body = [0u8; 8];
                stream.read_exact(&mut body)?;
                let vk = u16::from_le_bytes(body[0..2].try_into().unwrap());
                let _scan = u16::from_le_bytes(body[2..4].try_into().unwrap());
                let flags = u32::from_le_bytes(body[4..8].try_into().unwrap());

                // KEYEVENTF_KEYUP = 0x0002
                let down = (flags & 0x0002) == 0;
                let _ = backend_guard.key(vk, down);
            }
            _ => {}
        }
    }
    Ok(())
}
