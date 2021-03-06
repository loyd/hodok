use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;
use std::sync::mpsc;
use std::thread;

use byteorder::{BigEndian, WriteBytesExt};
use httparse;
use rustc_serialize::base64::{self, ToBase64};
use sha1::Sha1;

use base::node;
use constants::PORT;
use messages::{Attitude, VideoFrame, SysInfo};


fn get_mime(ext: &str) -> &'static str {
    match ext {
        "html" => "text/html",
        "js" => "application/javascript",
        "png" => "mage/png",
        "svg" => "image/svg+xml",
        _ => ""
    }
}

struct Handler {
    video: Option<TcpStream>,
    attitude: Option<TcpStream>,
    sysinfo: Option<TcpStream>
}

impl Handler {
    fn handle(&mut self, mut stream: TcpStream) {
        let (path, headers) = self.parse_req(&mut stream);

        debug!("handling request {}", path);

        if headers.contains_key("Upgrade") {
            self.handle_ws(stream, &path[1..], &headers.get("Sec-WebSocket-Key").unwrap());
        } else {
            self.handle_http(stream, &path[1..]);
        }
    }

    fn parse_req(&self, stream: &mut TcpStream) -> (String, HashMap<String, String>) {
        let mut data = [0; 4096];
        stream.read(&mut data).unwrap();

        let mut headers_buf = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers_buf);
        req.parse(&data).unwrap().unwrap();

        let path = req.path.unwrap_or("").to_string();
        let mut headers = HashMap::new();

        for header in req.headers {
            let name = header.name.to_string();
            let value = str::from_utf8(header.value).unwrap().to_string();
            headers.insert(name, value);
        }

        (path, headers)
    }

    fn handle_http(&self, mut stream: TcpStream, mut path: &str) {
        if path == "" {
            path = "index.html";
        }

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                self.send_404(stream);
                return;
            }
        };

        let ext = Path::new(path).extension().and_then(|x| x.to_str()).unwrap_or("");

        let mut header = String::new();
        header.push_str("HTTP/1.1 200 OK\r\nContent-Type: ");
        header.push_str(get_mime(ext));
        header.push_str("\r\n\r\n");

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        stream.write_all(header.as_bytes()).unwrap();
        stream.write_all(&data).unwrap();

        stream.write(b"\r\n\r\n").unwrap();
    }

    fn send_404(&self, mut stream: TcpStream) {
        stream.write(b"HTTP/1.1 404 Not Found\r\n").unwrap();
    }

    fn handle_ws(&mut self, mut stream: TcpStream, channel: &str, key: &str) {
        stream.write(b"HTTP/1.1 101 Switching Protocols\r\n\
                       Upgrade: websocket\r\n\
                       Connection: Upgrade\r\n\
                       Sec-WebSocket-Accept: ").unwrap();

        stream.write(self.compute_accept(key).as_bytes()).unwrap();
        stream.write(b"\r\n\r\n").unwrap();

        match channel {
            "video" => self.video = Some(stream),
            "attitude" => self.attitude = Some(stream),
            "sysinfo" => self.sysinfo = Some(stream),
            _ => {}
        }
    }

    fn compute_accept(&self, key: &str) -> String {
        // Step 1: appending.
        let mut pre_hash = key.to_string();
        pre_hash.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

        // Step 2: sha1.
        let mut hash = Sha1::new();
        hash.update(pre_hash.as_bytes());
        let mut out = [0u8; 20];
        hash.output(&mut out);

        // Step 3: base64.
        out.to_base64(base64::STANDARD)
    }

    fn send_video_frame(&mut self, frame: &VideoFrame) {
        let mut stream = self.video.take();

        if let Some(ref mut ws) = stream {
            self.send_ws(ws, &frame[..]);
        }

        self.video = stream;
    }

    fn send_attitude(&mut self, attitude: &Attitude) {
        let mut stream = self.attitude.take();

        if let Some(ref mut ws) = stream {
            let data: &[u8; 32] = unsafe { mem::transmute(attitude) };
            self.send_ws(ws, data);
        }

        self.attitude = stream;
    }

    fn send_sysinfo(&mut self, sysinfo: &SysInfo) {
        let mut stream = self.sysinfo.take();

        if let Some(ref mut ws) = stream {
            let data: &[u8; 7] = unsafe { mem::transmute(sysinfo) };
            self.send_ws(ws, data);
        }

        self.sysinfo = stream;
    }

    fn send_ws(&self, stream: &mut TcpStream, data: &[u8]) {
        let len = data.len();

        // Fin: 1, rsv: 0, opcode: 0x2 (binary).
        stream.write_u8(0b1_000_0010).unwrap();

        // Mask: 0.
        match len {
            0...125 => stream.write_u8(len as u8).unwrap(),
            126...65535 => {
                stream.write_u8(126).unwrap();
                stream.write_u16::<BigEndian>(len as u16).unwrap();
            },
            _ => {
                stream.write_u8(127).unwrap();
                stream.write_u64::<BigEndian>(len as u64).unwrap();
            }
        }

        stream.write(data).unwrap();
        stream.flush().unwrap();
    }
}

pub fn worker() {
    let video_frame_rx = node::subscribe::<VideoFrame>();
    let attitude_rx = node::subscribe::<Attitude>();
    let sys_info_rx = node::subscribe::<SysInfo>();

    let mut hander = Handler { video: None, attitude: None, sysinfo: None };

    let (tcp_tx, tcp_rx) = mpsc::channel();
    thread::spawn(move || {
        let addr = ("0.0.0.0", PORT);
        let listener = TcpListener::bind(addr).unwrap();

        info!("listening on {}:{}", addr.0, addr.1);

        for stream in listener.incoming() {
            tcp_tx.send(stream.unwrap()).unwrap();
        }
    });

    loop {
        select! {
            stream = tcp_rx.recv() => hander.handle(stream.unwrap()),
            frame = video_frame_rx.recv() => hander.send_video_frame(&*frame.unwrap()),
            attitude = attitude_rx.recv() => hander.send_attitude(&*attitude.unwrap()),
            sysinfo = sys_info_rx.recv() => hander.send_sysinfo(&*sysinfo.unwrap())
        }
    }
}
