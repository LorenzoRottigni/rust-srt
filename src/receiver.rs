use futures::StreamExt;
use opencv::{
    prelude::*,
    core::Vector,
    imgcodecs,
    highgui,
};
use srt_tokio::{SrtSocket, SrtSocketBuilder};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("[RECEIVER] Listening on 127.0.0.1:9000 ...");

    let builder = SrtSocketBuilder::default()
        .local_port(9000)
        .latency(Duration::from_millis(120));

    // This returns a single SrtSocket (the client connection)
    let mut socket: SrtSocket = SrtSocketBuilder::listen(builder).await?;
    println!("[RECEIVER] ✅ Client connected. Receiving frames...");

    loop {
        // recv() returns Result<(Instant, Bytes), Error>
        let next = socket.next().await;
        let (instant, data) = match next {
            Some(Ok((instant, data))) => (instant, data),
            Some(Err(e)) => {
                eprintln!("[RECEIVER] ❌ Stream error: {e}");
                break;
            }
            None => {
                eprintln!("[RECEIVER] ❌ Stream ended");
                break;
            }
        };

        println!("[RECEIVER] ✅ Received frame: {} bytes", data.len());

        // Decode JPEG to OpenCV Mat
        let mat = imgcodecs::imdecode(&Vector::from_slice(&data), imgcodecs::IMREAD_COLOR)?;
        if mat.empty() {
            eprintln!("[RECEIVER] ❌ Decoded empty frame");
            continue;
        }

        // Show frame
        highgui::imshow("SRT Stream", &mat)?;
        let key = highgui::wait_key(1)?;
        if key == 27 {
            println!("[RECEIVER] ESC pressed. Exiting.");
            break;
        }
    }

    Ok(())
}
