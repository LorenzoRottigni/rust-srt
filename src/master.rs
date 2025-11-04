use opencv::{
    core::Vector,
    highgui,
    imgcodecs,
    prelude::*,
};
use srt_tokio::SrtSocket;
use futures_util::stream::TryStreamExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Master: Listening on 0.0.0.0:3333...");
    let mut srt_socket = SrtSocket::builder().listen_on(":3333").await?;
    println!("Master: Waiting for slave connection...");

    let mut count = 0;
    while let Some((_instant, bytes)) = srt_socket.try_next().await? {
        count += 1;
        println!("Master: Received frame {count}, size {} bytes", bytes.len());

        if bytes.is_empty() {
            eprintln!("Master: Empty frame {count}");
            continue;
        }

        let buf = Vector::from_slice(&bytes);
        match imgcodecs::imdecode(&buf, imgcodecs::IMREAD_COLOR) {
            Ok(frame) => {
                println!("Master: Decoded frame {count}, {}x{}", frame.cols(), frame.rows());
                highgui::imshow("Master View", &frame)?;
                let key = highgui::wait_key(1)?;
                if key == 27 {
                    println!("Master: ESC pressed, exiting");
                    break;
                }
            }
            Err(e) => eprintln!("Master: Failed to decode frame {count}: {e}"),
        }
    }

    println!("Master: Connection closed");
    Ok(())
}
