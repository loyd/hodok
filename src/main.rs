#![allow(dead_code)]
#![feature(mpsc_select)]
extern crate byteorder;
extern crate httparse;
extern crate libc;
#[macro_use]
extern crate log;
extern crate rscam;
extern crate rustc_serialize;
extern crate sha1;

// Utils.
mod constants;
mod logger;
mod messages;
mod node;

// Nodes.
mod ahrs;
mod control;
mod server;
mod sysinfo;
mod video;


static NODES: &'static [fn()] = &[
    ahrs::worker,
    server::worker,
    sysinfo::worker,
    video::worker,
];

fn main() {
    logger::init().unwrap();

    node::run(&NODES);
}
