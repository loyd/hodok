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

use constants;
use messages::{Attitude, VideoFrame, SysInfo};
use nodes::{Node, Input};


pub struct Server {
    pub video_frame: Input<VideoFrame>,
    pub attitude: Input<Attitude>,
    pub sysinfo: Input<SysInfo>
}

struct Handler {
    video: Option<TcpStream>,
    attitude: Option<TcpStream>,
    sysinfo: Option<TcpStream>
}

impl Handler {
    fn handle(&mut self, mut stream: TcpStream) {
        let (path, headers) = self.parse_req(&mut stream);

        if headers.contains_key("Upgrade") {
            self.handle_ws(stream, &path, &headers.get("Sec-WebSocket-Key").unwrap());
        } else {
            self.handle_http(stream, &path);
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

    fn handle_http(&self, stream: TcpStream, path: &str) {
        match path {
            "/" | "/index.html" => self.send_file(stream, "index.html"),
            "/bundle.js" => self.send_file(stream, "bundle.js"),
            _ => self.send_404(stream)
        }
    }

    fn send_file(&self, mut stream: TcpStream, path: &str) {
        let mut res = String::new();
        let mut file = File::open(path).unwrap();
        let ext = Path::new(path).extension().and_then(|x| x.to_str());

        res.push_str("HTTP/1.1 200 OK\r\nContent-Type: ");
        res.push_str(match ext {
            Some("html") => "text/html",
            Some("js") => "application/javascript",
            _ => ""
        });
        res.push_str("; charset=utf-8\r\n\r\n");
        file.read_to_string(&mut res).unwrap();
        res.push_str("\r\n");

        stream.write(res.as_bytes()).unwrap();
    }

    fn send_404(&self, mut stream: TcpStream) {
        stream.write(b"HTTP/1.1 404 Not Found\r\n").unwrap();
    }

    fn handle_ws(&mut self, mut stream: TcpStream, path: &str, key: &str) {
        stream.write(b"HTTP/1.1 101 Switching Protocols\r\n\
                       Upgrade: websocket\r\n\
                       Connection: Upgrade\r\n\
                       Sec-WebSocket-Accept: ").unwrap();

        stream.write(self.compute_accept(key).as_bytes()).unwrap();
        stream.write(b"\r\n\r\n").unwrap();

        match path {
            "/video" => self.video = Some(stream),
            "/attitude" => self.attitude = Some(stream),
            "/sysinfo" => self.sysinfo = Some(stream),
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

impl Node for Server {
    fn new() -> Server {
        Server {
            video_frame: Input::new(),
            attitude: Input::new(),
            sysinfo: Input::new()
        }
    }

    fn main(&mut self) {
        println!("====================");

        let mut spriv = Handler { video: None, attitude: None, sysinfo: None };

        let (tx_tcp, rx_tcp) = mpsc::channel();
        thread::spawn(move || {
            let addr = ("0.0.0.0", constants::PORT);
            let listener = TcpListener::bind(addr).unwrap();

            for stream in listener.incoming() {
                tx_tcp.send(stream.unwrap()).unwrap();
            }
        });

        let rx_att = &self.attitude.1;
        let rx_vid = &self.video_frame.1;

        loop {
            select! {
                stream = rx_tcp.recv() => spriv.handle(stream.unwrap()),
                attitude = rx_att.recv() => spriv.send_attitude(&*attitude.unwrap()),
                frame = rx_vid.recv() => spriv.send_video_frame(&*frame.unwrap())
            }
        }
    }
}
