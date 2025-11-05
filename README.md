### Examples

https://github.com/russelltg/srt-rs/blob/main/srt-tokio/examples/

### WSL

open camera from powershell:

TCP: ffmpeg -rtbufsize 2000M -f dshow -i video="HD USB Camera" -f mpegts tcp://0.0.0.0:12345?listen=1

UDP: ffmpeg -f dshow -i video="HD USB Camera" -f mpegts udp://127.0.0.1:12345
