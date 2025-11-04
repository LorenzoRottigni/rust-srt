use opencv::{prelude::*, videoio, core::Vector, imgcodecs};
use srt_tokio::SrtSocket;
use bytes::Bytes;
use futures_util::sink::SinkExt;
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:2223";
    println!("Tenant connecting to {}", addr);

    // Only an optional stream_id is allowed (None = no stream_id)
    let mut socket = SrtSocket::builder()
        .call(addr, None) // ✅ just None
        .await
        .expect("Failed to connect to controller");

    println!("SRT handshake complete, starting camera...");

    // Open default camera
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        panic!("Cannot open camera");
    }

    let mut frame_count = 0;

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        if frame.empty() {
            continue;
        }

        // Encode frame to JPEG
        let mut buf = Vector::<u8>::new();
        imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::<i32>::new())?;

        frame_count += 1;
        println!("Sending frame {}: {} bytes", frame_count, buf.len());

        // Send (timestamp, bytes)
        socket.send((Instant::now(), Bytes::from(buf.to_vec()))).await?;

        sleep(Duration::from_millis(33)).await; // ~30 FPS
    }
}
