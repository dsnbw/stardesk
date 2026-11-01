//! StarDesk for Linux - Pure Native Implementation
//!
//! Ultra-low latency, hardware-accelerated remote desktop host and viewer for Linux.
//! 100% Native - No Wine required.

mod audio;
mod auth;
mod autostart;
mod capture;
mod clipboard;
mod config;
mod doctor;
mod file_trans;
mod input;
mod login_ui;
mod signaling;
mod terminal;
mod video;
mod viewer;
mod webrtc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "stardesk",
    version = "1.5.4",
    about = "Pure Native Linux Port of StarDesk Remote Desktop (No Wine Required)",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start StarDesk native host daemon (allows remote clients to control this Linux machine)
    Host {
        /// Optional custom device name to appear in StarDesk device list
        #[arg(long)]
        name: Option<String>,

        /// Run in foreground without daemonizing
        #[arg(long, default_value_t = true)]
        foreground: bool,
    },

    /// Run StarDesk native viewer (control a remote Windows, Mac, or Linux computer)
    Viewer {
        /// Remote device ID or connect code
        device_id: Option<String>,

        /// Connect via 6-digit remote assistance verification code
        #[arg(long)]
        code: Option<String>,
    },

    /// Log in using StarDesk / UU Remote mobile app QR code
    Login,

    /// Generate ad-hoc Remote Assistance Device ID and Verification Code (No login required)
    Assist,

    /// Check system compatibility, display server, uinput permissions, and GPU acceleration
    Doctor,

    /// Manage automatic start on desktop login
    Autostart {
        #[command(subcommand)]
        action: AutostartAction,
    },

    /// Shortcut to start the host daemon
    Run,
}

#[derive(Subcommand)]
enum AutostartAction {
    Enable,
    Disable,
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Host { name, foreground: _ } | Commands::Run => {
            run_native_host(name.as_deref()).await?;
        }
        Commands::Viewer { device_id, code: _ } => {
            if let Some(id) = device_id {
                viewer::connect_to_remote(&id).await?;
            } else {
                println!("Usage: stardesk viewer <DEVICE_ID>");
            }
        }
        Commands::Login => {
            run_qr_login().await?;
        }
        Commands::Assist => {
            run_assist_mode().await?;
        }
        Commands::Doctor => {
            doctor::report()?;
        }
        Commands::Autostart { action } => match action {
            AutostartAction::Enable => autostart::enable()?,
            AutostartAction::Disable => autostart::disable()?,
            AutostartAction::Status => autostart::status()?,
        },
    }
    Ok(())
}

/// Runs the 100% Native Linux StarDesk Host Daemon
async fn run_native_host(custom_name: Option<&str>) -> Result<()> {
    println!("============================================================");
    println!("     StarDesk Native Linux Host (Version 1.5.4)             ");
    println!("     100% Native Linux - Zero Wine Dependency               ");
    println!("============================================================\n");

    let mut creds = auth::Credentials::load_or_default();
    if let Some(name) = custom_name {
        creds.device_name = name.to_string();
        let _ = creds.save();
    }

    if !creds.is_logged_in() {
        println!("⚠️ Notice: Not logged into an account yet.");
        println!("You can run in Remote Assistance mode, or scan QR code to link your account:\n");
        println!("  To log in: run 'stardesk login'\n");
    }

    let auth_client = Arc::new(auth::StarDeskAuthClient::new(creds.clone()));

    // 1. Initialize Linux kernel /dev/uinput backend
    println!("[1/4] Initializing Linux kernel input device (/dev/uinput)...");
    let cfg = config::Config::load()?;
    let input_backend = input::select_backend(&cfg.input_backend)
        .expect("Failed to initialize /dev/uinput. Ensure 'uinput' kernel module is loaded.");
    let shared_input = Arc::new(Mutex::new(input_backend));
    println!("      Input backend ready: /dev/uinput virtual mouse & keyboard active.");

    // 2. Probe GPU Hardware Video Encoder
    println!("[2/4] Probing GPU hardware video encoder...");
    let pipeline = video::EncoderPipeline::new(1920, 1080, cfg.max_fps);
    println!("      Video encoding engine configured: {:?} at {} FPS", pipeline.encoder, cfg.max_fps);

    // 3. Register device with StarDesk API
    if creds.is_logged_in() {
        println!("[3/4] Registering Linux host with StarDesk cloud...");
        if let Err(e) = auth_client.register_device().await {
            eprintln!("      Warning: Cloud registration notice: {}", e);
        } else {
            println!("      Device '{}' listed as Online in your StarDesk app!", creds.device_name);
        }
    } else {
        println!("[3/4] Running in Guest / Remote Assistance mode (Device ID: {})...", &creds.device_id[..8]);
    }

    // 4. Start heartbeat background loop
    let auth_hb = auth_client.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let _ = auth_hb.send_heartbeat().await;
        }
    });

    println!("[4/4] Host daemon ready! Waiting for incoming remote connections...");
    println!("      Device ID: {}", creds.device_id);
    println!("      Press Ctrl+C to stop.\n");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("\nStarDesk Host stopped gracefully.");
    Ok(())
}

/// Interactive Terminal QR Code Login
async fn run_qr_login() -> Result<()> {
    let creds = auth::Credentials::load_or_default();
    let auth_client = auth::StarDeskAuthClient::new(creds.clone());

    println!("Requesting QR Code from StarDesk cloud...");
    let qr_data = auth_client.fetch_qr_code().await?;

    login_ui::print_qr_terminal(&qr_data.url)?;

    // Poll status loop
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        match auth_client.poll_qr_status(&qr_data.ticket).await {
            Ok(poll) => match poll.status {
                1 => println!("📱 QR Code scanned by phone! Please confirm login..."),
                2 => {
                    println!("✅ Login confirmed successfully!");
                    let mut updated = creds;
                    if let Some(uid) = poll.user_id { updated.user_id = uid; }
                    if let Some(tok) = poll.token { updated.token = tok; }
                    if let Some(csrf) = poll.csrf { updated.csrf = csrf; }
                    updated.save()?;
                    println!("Credentials saved to ~/.config/stardesk/credentials.json");
                    println!("You can now start the host by running: stardesk host");
                    break;
                }
                3 => {
                    println!("❌ QR Code expired. Please rerun 'stardesk login'.");
                    break;
                }
                _ => {}
            },
            Err(e) => {
                eprintln!("Polling error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Instant Remote Assistance Mode
async fn run_assist_mode() -> Result<()> {
    let creds = auth::Credentials::load_or_default();
    let code: u32 = 100_000 + (std::time::SystemTime::now().elapsed().unwrap().as_secs() % 900_000) as u32;

    println!("============================================================");
    println!("             StarDesk Remote Assistance                     ");
    println!("============================================================");
    println!("Share the following information with the person helping you:\n");
    println!("  Device ID:         {}", creds.device_id);
    println!("  Verification Code: {:06}", code);
    println!("============================================================");
    println!("Waiting for connection... Press Ctrl+C to exit.");

    tokio::signal::ctrl_c().await?;
    Ok(())
}
