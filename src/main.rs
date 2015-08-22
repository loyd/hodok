#![allow(dead_code)]
#![feature(mpsc_select)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate httparse;
extern crate libc;
extern crate rscam;
extern crate rustc_serialize;
extern crate sha1;

#[macro_use]
mod node;
mod constants;
mod logger;
mod messages;


fn main() {
    logger::init().unwrap();

    run_nodes![
        ahrs
        control
        server
        sysinfo
        video
    ];
}
