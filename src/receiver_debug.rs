// receiver.rs
use srt_tokio::SrtSocket;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Receiver listening on :1234");
    let mut socket = SrtSocket::builder()
        .latency(Duration::from_millis(1000))
        .listen_on(":1234")   // server side: listen here
        .await?;
    println!("Connection established");

    let mut file = tokio::fs::File::create("received.ts").await?;
    let mut frame_index = 0u64;

    while let Some(item) = socket.next().await {
        match item {
            Ok((instant, bytes)) => {
                println!("Received frame {} at {:?}, size {}", frame_index, instant, bytes.len());
                file.write_all(&bytes).await?;
                frame_index += 1;
            }
            Err(e) => {
                eprintln!("Error receiving frame {}: {:?}", frame_index, e);
                break;
            }
        }
    }

    println!("Receiver: done, received {} frames", frame_index);
    Ok(())
}
