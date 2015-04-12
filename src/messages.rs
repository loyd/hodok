pub use rscam::Frame as VideoFrame;

pub struct Attitude(pub f32, pub f32, pub f32, pub f32);

pub struct SysInfo {
    pub free_mem: u8,
    pub avail_mem: u8,
    pub cpu: u8,
    pub loadavg: (u8, u8, u8)
}
