pub const AHRS_I2C_BUS: u32 = 1;
pub const AHRS_RATE: f32 = 30.;     // [Hz]
pub const ACCEL_RANGE: f32 = 2.;    // [g]
pub const MAGN_RANGE: f32 = 4.;     // [Gauss]
pub const GYRO_RANGE: f32 = 250.;   // [°/s]

pub const PORT: u16 = 8000;

pub const VIDEO_PATH: &'static str = "/dev/video0";
pub const VIDEO_FPS: u32 = 10;
pub const VIDEO_RESOL: (u32, u32) = (640, 480);
