use bytes::Bytes;
use futures::{stream, SinkExt};
use srt_tokio::SrtSocket;
use std::{io, time::Duration, time::Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Client connecting to 127.0.0.1:2223...");

    let mut tx = SrtSocket::builder()
        .latency(Duration::from_millis(120))
        .call("127.0.0.1:2223", None)
        .await
        .expect("Failed to connect to server");

    let messages = ["hello", "world", "camera"];

    for msg in messages {
        println!("Sending: {}", msg);
        tx.send((Instant::now(), Bytes::from(msg))).await?;
        sleep(Duration::from_millis(50)).await;
    }

    // Wait to ensure server has time to read
    sleep(Duration::from_secs(1)).await;

    tx.close().await?;
    println!("Client finished sending.");
    Ok(())
}
