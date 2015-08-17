#![allow(dead_code)]
#![feature(mpsc_select)]
extern crate byteorder;
extern crate httparse;
extern crate libc;
extern crate rscam;
extern crate rustc_serialize;
extern crate sha1;

mod math;
mod ifaces;
mod periphery;
mod constants;
mod messages;
mod nodes;

use nodes::Node;


fn main() {
    let mut ahrs = nodes::ahrs::Ahrs::new();
    let server = nodes::server::Server::new();
    let mut sysinfo = nodes::sysinfo::SysInformer::new();
    let mut video = nodes::video::Video::new();

    ahrs.attitude.pipe(&server.attitude);
    video.video_frame.pipe(&server.video_frame);
    sysinfo.info.pipe(&server.sysinfo);

    ahrs.start();
    server.start();
    sysinfo.start();
    video.start().join();
}
