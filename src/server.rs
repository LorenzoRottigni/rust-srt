use futures::TryStreamExt;
use srt_tokio::SrtSocket;
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("SRT server listening on 0.0.0.0:2223");

    loop {
        println!("Waiting for a new SRT connection...");

        let mut rx = SrtSocket::builder()
            .latency(Duration::from_millis(120))
            .listen_on(2223)
            .await
            .expect("Failed to bind SRT");

        println!("Client connected!");

        while let Ok(Some((_ts, data))) = rx.try_next().await {
            println!("Received: {}", String::from_utf8_lossy(&data));
        }

        println!("Client disconnected — still listening...");
    }
}
