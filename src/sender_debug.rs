// sender.rs
use ac_ffmpeg::{
    format::{
        demuxer::Demuxer,
        io::IO,
        muxer::{Muxer, OutputFormat},
    },
    time::Timestamp,
};
use bytes::Bytes;
use futures::SinkExt;
use srt_tokio::SrtSocket;
use tokio::{sync::mpsc::channel, time::sleep_until};
use tokio_stream::StreamExt;
use std::{
    fs::File,
    io::{self, Write},
    time::{Duration, Instant},
};

/// Bridges FFmpeg output to a Tokio MPSC channel for async SRT sending.
struct WriteBridge(tokio::sync::mpsc::Sender<(Instant, Bytes)>);
impl Write for WriteBridge {
    fn write(&mut self, w: &[u8]) -> io::Result<usize> {
        for chunk in w.chunks(1316) {
            if self.0.try_send((Instant::now(), Bytes::copy_from_slice(chunk))).is_err() {
                println!("⚠️ Buffer full, dropping packet");
            }
        }
        Ok(w.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Open input file and initialize demuxer ---
    let input = File::open("video.mp4")?;
    let io_read = IO::from_seekable_read_stream(input);
    let mut demuxer = Demuxer::builder()
        .build(io_read)?
        .find_stream_info(None)
        .map_err(|(_, e)| e)?;
    
    let streams = demuxer.streams();
    if streams.is_empty() {
        anyhow::bail!("❌ No streams found in input file!");
    }
    println!("🎥 Found {} stream(s) in input", streams.len());

    // --- Connect to receiver over SRT ---
    println!("Sender connecting to receiver …");
    let mut socket = SrtSocket::builder()
        .latency(Duration::from_millis(1000))
        .call("127.0.0.1:1234", None) // client mode
        .await?;
    println!("✅ Connected to receiver!");

    let (tx, rx) = channel(1024);
    let mut last_pts_inst: Option<(Timestamp, Instant)> = None;

    // --- Spawn demuxer + muxer task ---
    let demux_task = tokio::spawn(async move {
        let io_write = IO::from_write_stream(WriteBridge(tx));
        let mut muxer_builder = Muxer::builder();

        // Add all streams to muxer
        for stream in demuxer.streams() {
            let params = stream.codec_parameters();
            muxer_builder
                .add_stream(&params)
                .expect("Failed to add stream to muxer");
        }

        let mut muxer = muxer_builder
            .build(io_write, OutputFormat::find_by_name("mpegts").unwrap())
            .expect("Failed to build muxer");

        println!("📦 Muxer ready, starting streaming loop");

        while let Some(packet) = demuxer.take()? {
            let pts = packet.pts();
            let inst = match last_pts_inst {
                Some((last_pts, last_inst)) if pts >= last_pts => {
                    let delta = pts - last_pts;
                    let deadline = last_inst + delta;
                    sleep_until(deadline.into()).await;
                    last_pts_inst = Some((pts, deadline));
                    deadline
                }
                _ => {
                    let now = Instant::now();
                    last_pts_inst = Some((pts, now));
                    now
                }
            };
            println!("⏱️ Sending packet @ {:?} ({} bytes)", inst, packet.data().len());
            muxer.push(packet)?;
        }

        println!("✅ Finished muxing all packets");
        Ok::<(), anyhow::Error>(())
    });

    // --- Send muxed TS packets over SRT ---
    let mut stream = tokio_stream::wrappers::ReceiverStream::new(rx).map(Ok::<_, io::Error>);
    socket.send_all(&mut stream).await?;

    demux_task.await??;
    println!("🏁 Sender finished");
    Ok(())
}
