use opencv::{core::Vector, highgui, imgcodecs, prelude::*};
use srt_tokio::SrtSocket;
use futures::stream::StreamExt;
use tokio::time::{sleep, Duration};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> opencv::Result<()> {
    let port = 2223;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("Controller listening on {}", addr);

    // Listen on TCP-like SRT socket
    let mut socket = SrtSocket::builder()
        .listen_on(addr)
        .await
        .expect("Failed to listen on SRT socket");

    println!("SRT handshake complete, waiting for frames...");

    while let Some(frame_res) = socket.next().await {
        match frame_res {
            Ok((_ts, bytes)) => {
                println!("Received frame: {} bytes", bytes.len());

                let buf: Vec<u8> = bytes.to_vec();
                let vec_u8 = Vector::<u8>::from_iter(buf);
                let mat = imgcodecs::imdecode(&vec_u8, imgcodecs::IMREAD_COLOR)?;
                highgui::imshow("Tenant Camera", &mat)?;
                if highgui::wait_key(1)? == 27 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving frame: {e}");
            }
        }

        sleep(Duration::from_millis(1)).await;
    }

    Ok(())
}
