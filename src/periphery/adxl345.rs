use std::convert::From;
use std::io::Error as IoError;
use std::num::Float;
use std::result;

use ifaces::I2C;


pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Unidentified
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

pub struct Adxl345 {
    underline: I2C,
    running: bool,
    gain: f32,
    buf: [u8; 6]
}

impl Adxl345 {
    pub fn new(i2c_bus: u32) -> Result<Adxl345> {
        Adxl345::with_addr(i2c_bus, 0x53)
    }

    pub fn with_addr(i2c_bus: u32, addr: u16) -> Result<Adxl345> {
        let underline = try!(I2C::open(i2c_bus, addr));
        try!(Adxl345::identify(&underline));
        Ok(Adxl345 {
            underline: underline,
            running: false,
            gain: Float::nan(),
            buf: [0; 6]
        })
    }

    fn identify(i2c: &I2C) -> Result<()> {
        let mut check = [0];
        try!(i2c.read(0x00, &mut check));
        if check[0] != 0xe5 { Err(Error::Unidentified) } else { Ok(()) }
    }

    pub fn set_rate(&mut self, expected: f32) -> Result<f32> {
        static RATES: [f32; 16] = [0.1, 0.2, 0.39, 0.78, 1.56, 3.13, 6.25,
                                   12.5, 25., 50., 100., 200., 400., 800., 1600., 3200.];
                            //    |<~~~~~~~ Low power mode. ~~~~~~>|

        let mut ctl = RATES.iter().position(|x| expected <= *x).unwrap_or(15);
        let actual = RATES[ctl];

        if 12.5 <= actual && actual <= 400.0 { ctl |= 0x10; }

        self.buf[0] = 0x2c;
        self.buf[1] = ctl as u8;

        try!(self.underline.write(&self.buf[0..2]));
        Ok(actual)
    }

    pub fn set_range(&mut self, expected: f32) -> Result<f32> {
        static RANGES: [f32; 4] = [2., 4., 8., 16.];

        let ctl = RANGES.iter().position(|x| expected <= *x).unwrap_or(3);
        let actual = RANGES[ctl];

        self.buf[0] = 0x31;
        self.buf[1] = ctl as u8 | 0x08;   // Full resolution mode.

        self.gain = actual/((512 << ctl) as f32);

        try!(self.underline.write(&self.buf[0..2]));
        Ok(actual)
    }

    pub fn start(&mut self) -> Result<()> {
        self.buf[0] = 0x2d;
        self.buf[1] = 0x08;
        try!(self.underline.write(&self.buf[0..2]));
        self.running = true;
        Ok(())
    }

    pub fn measure(&mut self) -> Result<(f32, f32, f32)> {
        try!(self.underline.read(0x32, &mut self.buf));
        Ok((
            ((self.buf[1] as i16) << 8 | (self.buf[0] as i16)) as f32 * self.gain,
            ((self.buf[3] as i16) << 8 | (self.buf[2] as i16)) as f32 * self.gain,
            ((self.buf[5] as i16) << 8 | (self.buf[4] as i16)) as f32 * self.gain
        ))
    }

    pub fn stop(&mut self) -> Result<()> {
        self.buf[0] = 0x2d;
        self.buf[1] = 0x00;
        try!(self.underline.write(&self.buf[0..2]));
        self.running = false;
        Ok(())
    }
}

impl Drop for Adxl345 {
    fn drop(&mut self) {
        if self.running {
            let _ = self.stop();
        }
    }
}
