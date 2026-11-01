use anyhow::Result;
use qrcode::QrCode;
use qrcode::render::unicode;

pub fn print_qr_terminal(url: &str) -> Result<()> {
    println!("\n=== StarDesk Mobile App Login ===");
    println!("Scan this QR Code using the StarDesk or UU Remote Mobile App:");
    println!("Or open link: {}\n", url);

    if let Ok(code) = QrCode::new(url.as_bytes()) {
        let image = code.render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        println!("{}", image);
    } else {
        println!("Visit URL to log in: {}", url);
    }

    println!("\nWaiting for confirmation on your phone...\n");
    Ok(())
}
