use srt_tokio::SrtSocket;
use std::io::Error;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to the streamer at 127.0.0.1:3333
    let mut srt_socket = SrtSocket::builder()
        .call("127.0.0.1:1234", None)
        .await?;

    println!("Connected to SRT streamer at 127.0.0.1:1234");

    let mut count = 0;

    // Receive packets
    while let Some((_instant, _bytes)) = srt_socket.try_next().await? {
        count += 1;
        print!("\rReceived {count} packets");
    }

    println!("\nConnection closed");

    Ok(())
}
