use bytes::Bytes;
use futures::{stream, SinkExt, StreamExt};
use opencv::{
    core,
    prelude::*,
    videoio,
    imgcodecs,
};
use srt_tokio::SrtSocket;
use std::{error::Error, sync::Arc};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Slave: Opening camera...");
    let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        panic!("Slave: Cannot open camera");
    }
    println!("Slave: Camera opened.");
    let cam = Arc::new(Mutex::new(cam));

    println!("Slave: Connecting to master at 127.0.0.1:3333...");
    let mut srt_socket = SrtSocket::builder().call("127.0.0.1:3333", None).await?;
    println!("Slave: Connected, streaming frames...");

    let cam_stream = cam.clone();
    let mut frame_stream = stream::unfold(0, move |count| {
        let cam_stream = cam_stream.clone();
        async move {
            let mut frame = Mat::default();
            let mut cam_lock = cam_stream.lock().await;
            let read_ok = cam_lock.read(&mut frame).unwrap_or(false);

            if read_ok && !frame.empty() {
                // Encode frame as JPEG
                let mut buf = core::Vector::new();
                let encode_ok = imgcodecs::imencode(".jpg", &frame, &mut buf, &core::Vector::new())
                    .unwrap_or(false);
                if !encode_ok {
                    eprintln!("Slave: Failed to encode frame {count}");
                    return None;
                }

                let bytes = Bytes::from(buf.to_vec());
                println!("Slave: Sending frame {count}, size {} bytes", bytes.len());

                sleep(Duration::from_millis(30)).await; // ~30 FPS
                Some((Ok((Instant::now(), bytes)), count + 1))
            } else {
                eprintln!("Slave: Failed to read frame {count} or frame empty");
                None
            }
        }
    }).boxed();

    srt_socket.send_all(&mut frame_stream).await?;
    Ok(())
}
