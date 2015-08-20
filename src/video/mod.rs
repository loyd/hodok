use std::default::Default;

use rscam::{Camera, Config};

use constants::{VIDEO_DEVICE, VIDEO_FPS, VIDEO_RESOLUTION};
use messages::VideoFrame;
use node;


pub fn worker() {
    let video_frame_tx = node::advertise::<VideoFrame>();

    let mut camera = Camera::new(VIDEO_DEVICE).unwrap();

    ::std::thread::sleep_ms(10_000);
    println!(">>>> video start");

    camera.start(&Config {
        interval: (1, VIDEO_FPS),
        resolution: VIDEO_RESOLUTION,
        format: b"H264",
        ..Default::default()
    }).unwrap();

    loop {
        let frame = camera.capture().unwrap();
        video_frame_tx.send(frame);
    }
}
