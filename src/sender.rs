use opencv::{
    prelude::*,
    videoio,
    imgcodecs,
    core,
};
use srt_tokio::SrtSocket;
use tokio::time::{sleep, Duration};
use futures_util::sink::SinkExt; // for send

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("[SENDER] Connecting to FFmpeg UDP stream...");

    // Open the UDP stream sent by ffmpeg.exe on Windows
    let stream_url = "tcp://172.19.96.1:12345";
    let mut cap = videoio::VideoCapture::from_file(stream_url, videoio::CAP_FFMPEG)?;

    if !videoio::VideoCapture::is_opened(&cap)? {
        panic!("[SENDER] Cannot open UDP stream from FFmpeg: {}", stream_url);
    }

    println!("[SENDER] Connecting to SRT receiver at 127.0.0.1:9000 ...");

    // Build and connect the SRT socket (call = client)
    let mut socket: SrtSocket = SrtSocket::builder()
        .call("127.0.0.1:9000", None)
        .await?;

    println!("[SENDER] Connected. Streaming frames...");

    loop {
        let mut frame = Mat::default();
        cap.read(&mut frame)?;
        if frame.empty() {
            eprintln!("[SENDER] Empty frame, skipping...");
            sleep(Duration::from_millis(10)).await;
            continue;
        }

        // Encode frame as JPEG
        let mut buf = core::Vector::<u8>::new();
        imgcodecs::imencode(".jpg", &frame, &mut buf, &core::Vector::new())?;

        // Send via SRT
        socket.send((tokio::time::Instant::now().into(), buf.to_vec().into())).await?;

        println!("[SENDER] ✅ Sent frame, size = {} bytes", buf.len());

        // ~30 FPS
        sleep(Duration::from_millis(33)).await;
    }
}
