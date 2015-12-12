use std::default::Default;

use rscam::{Camera, Config};
use rscam::{CID_MPEG_VIDEO_H264_PROFILE, MPEG_VIDEO_H264_PROFILE_BASELINE};
use rscam::{CID_MPEG_VIDEO_H264_I_PERIOD, CID_MPEG_VIDEO_REPEAT_SEQ_HEADER, CID_HFLIP, CID_VFLIP};

use base::node;
use constants::{VIDEO_DEVICE, VIDEO_FPS, VIDEO_RESOLUTION, VIDEO_GOF_SIZE};
use messages::VideoFrame;


pub fn worker() {
    let video = node::advertise::<VideoFrame>();

    let mut camera = Camera::new(VIDEO_DEVICE).unwrap();

    camera.set_control(CID_MPEG_VIDEO_H264_PROFILE, MPEG_VIDEO_H264_PROFILE_BASELINE).unwrap();
    camera.set_control(CID_MPEG_VIDEO_H264_I_PERIOD, VIDEO_GOF_SIZE).unwrap();
    camera.set_control(CID_MPEG_VIDEO_REPEAT_SEQ_HEADER, true).unwrap();
    camera.set_control(CID_HFLIP, true).unwrap();
    camera.set_control(CID_VFLIP, true).unwrap();

    camera.start(&Config {
        interval: (1, VIDEO_FPS),
        resolution: VIDEO_RESOLUTION,
        format: b"H264",
        ..Default::default()
    }).unwrap();

    loop {
        let frame = camera.capture().unwrap();
        video.send(frame);
    }
}
