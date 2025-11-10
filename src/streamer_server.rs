use std::{
    env,
    fs::File,
    io,
    time::{Duration, Instant},
};

use ac_ffmpeg::{
    format::{demuxer::Demuxer, io::IO, muxer::{Muxer, OutputFormat}},
    time::Timestamp,
};
use bytes::Bytes;
use futures::stream::iter;
use futures::SinkExt;
use srt_tokio::SrtSocket;
use tokio::{
    sync::mpsc::{channel, Sender},
    time::sleep_until,
};
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <video.mp4>", args[0]);
        return Ok(());
    }
    let video_path = &args[1];

    println!("Waiting for connection...");
    let mut socket = SrtSocket::builder()
        .latency(Duration::from_millis(1000))
        .listen_on(":1234")
        .await?;
    println!("Connection established");

    loop {
        // Open video file
        let input = File::open(video_path)?;
        let io = IO::from_seekable_read_stream(input);
        let mut demuxer = Demuxer::builder()
            .build(io)?
            .find_stream_info(None)?;

        // Create channel to send packets
        let (tx, rx) = channel::<(Instant, Bytes)>(1024);
        let tx_clone = tx.clone();

        // Spawn task to read from demuxer and push to channel
        tokio::spawn(async move {
            let streams = demuxer.streams()
                .iter()
                .map(|s| s.codec_parameters())
                .collect::<Vec<_>>();

            let io = IO::from_write_stream(SrtWriteBridge { sender: tx_clone });

            let mut muxer_builder = Muxer::builder();
            for params in streams {
                muxer_builder.add_stream(&params).unwrap();
            }
            let mut muxer = muxer_builder
                .build(io, OutputFormat::find_by_name("mpegts").unwrap())
                .unwrap();

            let mut last_pts_inst: Option<(f64, Instant)> = None;

            while let Some(packet) = demuxer.take().unwrap() {
                let pts_sec = packet.pts().as_f64(); // convert to seconds for simple math

                if let Some((last_pts, last_inst)) = last_pts_inst {
                    if pts_sec > last_pts {
                        let delta = pts_sec - last_pts;
                        let deadline = last_inst + Duration::from_secs_f64(delta);
                        sleep_until(deadline.into()).await;
                        last_pts_inst = Some((pts_sec, deadline));
                    }
                } else {
                    last_pts_inst = Some((pts_sec, Instant::now()));
                }

                muxer.push(packet).unwrap();
            }
        });

        // Send the packets over SRT
        let mut stream = ReceiverStream::new(rx).map(|r| r.map(|(_inst, bytes)| bytes));
        socket.send_all(&mut stream).await?;

        println!("Finished file, looping again...");
    }
}

// Bridge to convert write to SRT channel
struct SrtWriteBridge {
    sender: Sender<(Instant, Bytes)>,
}

impl io::Write for SrtWriteBridge {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for chunk in buf.chunks(1316) {
            let _ = self.sender.try_send((Instant::now(), Bytes::copy_from_slice(chunk)));
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
