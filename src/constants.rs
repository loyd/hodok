pub const AHRS_DEVICE: &'static str = "/dev/i2c-1";
pub const AHRS_RATE: f32 = 25.;     // [Hz]
pub const ACCEL_RANGE: f32 = 2.;    // [g]
pub const MAGN_RANGE: f32 = 4.;     // [Gauss]
pub const GYRO_RANGE: f32 = 250.;   // [°/s]

pub const PORT: u16 = 8000;

pub const VIDEO_DEVICE: &'static str = "/dev/video0";
pub const VIDEO_FPS: u32 = 20;
pub const VIDEO_RESOLUTION: (u32, u32) = (640, 480);
pub const VIDEO_GOF_SIZE: u32 = 120;

pub const SYSINFO_RATE: f32 = 2.;   // [Hz]
