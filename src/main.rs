#![feature(libc, std_misc, old_io, str_words)]
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
    // let sysinfo = nodes::sysinfo::SysInformer::new();
    // let server = nodes::server::Server::new();
    // let mut video = nodes::video::Video::new();
    // let mut ahrs = nodes::ahrs::Ahrs::new();

    // ahrs.attitude.pipe(&server.attitude);
    // video.video_frame.pipe(&server.video_frame);

    // ahrs.start();
    // video.start();
    // sysinfo.start();
    // server.start().join();
}
