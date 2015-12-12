pub use self::i2c::I2C;
pub use self::serial::Serial;


macro_rules! check_io(
    ($cond:expr) =>
        (if !$cond { return Err(io::Error::last_os_error()); })
);

pub mod i2c;
pub mod serial;
