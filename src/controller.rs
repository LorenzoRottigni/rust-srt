use futures::StreamExt;
use opencv::prelude::MatExprTraitConst;
use opencv::{
    core::{self, Mat, Point, Scalar},
    highgui, imgcodecs,
    imgproc::{put_text, FONT_HERSHEY_SIMPLEX, LINE_AA},
};
use srt_tokio::SrtSocket;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

fn main() -> opencv::Result<()> {
    println!("Controller starting…");

    let (tx_frame, mut rx_frame) = mpsc::channel::<Vec<u8>>(32);

    // Spawn network thread
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            // Listen mode
            let mut socket = SrtSocket::builder()
                .latency(Duration::from_millis(120))
                .listen_on(2223)
                .await
                .expect("SRT bind failed");

            println!("Waiting for tenant...");

            // According to docs, SrtSocket in listen mode implements Stream of (Instant, Bytes)
            while let Some(Ok((_ts, data))) = socket.next().await {
                if tx_frame.send(data.to_vec()).await.is_err() {
                    eprintln!("UI closed, stopping network thread");
                    break;
                }
            }

            println!("❌ Tenant disconnected or socket closed");
        });
    });

    // GUI (simplified)
    highgui::named_window("SRT Stream", highgui::WINDOW_AUTOSIZE)?;
    println!("Waiting for frames…");

    let mut last_frame_time = Instant::now();
    loop {
        let mat = match rx_frame.try_recv() {
            Ok(buf) => {
                last_frame_time = Instant::now();
                let cv_buf = core::Vector::<u8>::from_slice(&buf);
                match imgcodecs::imdecode(&cv_buf, imgcodecs::IMREAD_COLOR) {
                    Ok(m) => m,
                    Err(_) => {
                        let mut img = Mat::zeros(480, 640, core::CV_8UC3)?.to_mat()?;
                        put_text(
                            &mut img,
                            "No stream attached",
                            Point::new(50, 200),
                            FONT_HERSHEY_SIMPLEX,
                            1.0,
                            Scalar::new(255., 255., 255., 0.),
                            2,
                            LINE_AA,
                            false,
                        )?;
                        img
                    }
                }
            }
            Err(_) => {
                let mut img = Mat::zeros(480, 640, core::CV_8UC3)?.to_mat()?;
                let text = if last_frame_time.elapsed() > Duration::from_secs(2) {
                    "No tenant attached"
                } else {
                    "Waiting for frame..."
                };
                put_text(
                    &mut img,
                    text,
                    Point::new(50, 200),
                    FONT_HERSHEY_SIMPLEX,
                    1.0,
                    Scalar::new(255., 255., 255., 0.),
                    2,
                    LINE_AA,
                    false,
                )?;
                img
            }
        };

        highgui::imshow("SRT Stream", &mat)?;
        let key = highgui::wait_key(30)?;
        if key == 27 {
            break;
        }
    }

    println!("Stream ended.");
    Ok(())
}
