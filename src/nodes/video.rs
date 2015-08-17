use std::default::Default;

use rscam::{Camera, Config};

use constants;
use messages::VideoFrame;
use nodes::{Node, Output};


pub struct Video {
    pub video_frame: Output<VideoFrame>,
}

impl Node for Video {
    fn new() -> Video {
        Video {
            video_frame: Output::new()
        }
    }

    fn main(&mut self) {
        let mut camera = Camera::new(constants::VIDEO_DEVICE).unwrap();

        ::std::thread::sleep_ms(10_000);
        println!(">>>> video start");

        camera.start(&Config {
            interval: (1, constants::VIDEO_FPS),
            resolution: constants::VIDEO_RESOLUTION,
            format: b"H264",
            ..Default::default()
        }).unwrap();

        loop {
            let frame = camera.capture().unwrap();
            self.video_frame.send(frame);
        }
    }
}
