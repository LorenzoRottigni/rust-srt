use futures::SinkExt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use bytes::Bytes;
use srt_tokio::SrtSocket;
use anyhow::Result;
use std::time::Instant;

const FRAME_CHUNK_SIZE: usize = 1024 * 256; // e.g., 256KB chunks
const FRAME_INTERVAL_MS: u64 = 33;           // ~30fps

#[tokio::main]
async fn main() -> Result<()> {
    println!("Sender: binding …");
    let mut socket = SrtSocket::builder()
        .listen_on(2223)
        .await?;
    println!("Sender: client connected, starting frame stream …");

    let mut file = File::open("video.mp4").await?;
    let mut buf = vec![0u8; FRAME_CHUNK_SIZE];
    let mut frame_index: u64 = 0;

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            println!("Sender: end‐of‐file, sent {} frames", frame_index);
            break;
        }
        let data = &buf[..n];
        let bytes = Bytes::copy_from_slice(data);
        let now = Instant::now();

        // Send single “frame” as one message
        socket.send_all(
            &mut futures::stream::iter(std::iter::once(Ok((now, bytes))))
        ).await?;

        println!("Sender: sent frame {} ({} bytes)", frame_index, n);
        frame_index += 1;

        // wait for next frame interval
        tokio::time::sleep(tokio::time::Duration::from_millis(FRAME_INTERVAL_MS)).await;
    }

    // give some time for receiver to catch up
    println!("Sender: sleeping briefly before closing …");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    socket.close().await?;
    println!("Sender: closed socket.");
    Ok(())
}
