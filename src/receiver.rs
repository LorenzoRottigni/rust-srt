// receiver.rs
use srt_tokio::SrtSocket;
use futures::TryStreamExt;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Receiver: attempting to connect to sender …");

    // Retry loop
    let mut socket = loop {
        match SrtSocket::builder()
            .call("127.0.0.1:2223", None)
            .await
        {
            Ok(sock) => {
                println!("Receiver: connected!");
                break sock;
            }
            Err(e) => {
                eprintln!("Receiver: connect failed: {:?}. Retrying in 1s …", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    };

    println!("Receiver: awaiting frames …");
    let mut frame_index: u64 = 0;

    loop {
        match socket.try_next().await {
            Ok(Some((_instant, bytes))) => {
                let size = bytes.len();
                println!("Receiver: got frame {} ({} bytes)", frame_index, size);
                frame_index += 1;
            }
            Ok(None) => {
                println!("Receiver: connection closed by sender after {} frames", frame_index);
                break;
            }
            Err(e) => {
                println!("Receiver: error receiving at frame {}: {:?}", frame_index, e);
                break;
            }
        }
    }

    println!("Receiver: done.");
    Ok(())
}
