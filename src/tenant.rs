use bytes::Bytes;
use futures_util::sink::SinkExt; // Needed for `send`
use opencv::{core, highgui, imgcodecs, prelude::*, videoio};
use srt_tokio::SrtSocket;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> opencv::Result<()> {
    println!("Tenant starting…");

    // Connect to controller
    let mut socket = match SrtSocket::builder()
        .latency(Duration::from_millis(120))
        .call("127.0.0.1:2223", None)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Failed to connect to controller: {}", e);
            return Ok(());
        }
    };
    println!("✅ Connected to controller");

    // Open default camera
    let mut cam = match videoio::VideoCapture::new(0, videoio::CAP_ANY) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to open camera: {}", e);
            return Ok(());
        }
    };
    if !videoio::VideoCapture::is_opened(&cam)? {
        eprintln!("❌ Camera is not opened");
        return Ok(());
    }

    loop {
        let mut frame = core::Mat::default();
        if !cam.read(&mut frame)? {
            continue;
        }
        if frame.empty() {
            continue;
        }

        // Encode as JPEG
        let mut buf = core::Vector::<u8>::new();
        if imgcodecs::imencode(".jpg", &frame, &mut buf, &core::Vector::new()).is_err() {
            eprintln!("❌ Failed to encode frame");
            continue;
        }

        // Send over SRT as (Instant, Bytes)
        let data = Bytes::from(buf.to_vec());
        if let Err(e) = socket.send((Instant::now(), data)).await {
            eprintln!("❌ Failed to send frame: {}", e);
            break;
        }

        // Optional local preview
        if highgui::imshow("Tenant preview", &frame).is_err() {
            eprintln!("❌ Failed to show preview");
        }
        if highgui::wait_key(1)? == 27 {
            break; // ESC to quit
        }

        sleep(Duration::from_millis(33)).await; // ~30 FPS
    }

    println!("Tenant exiting.");
    Ok(())
}
