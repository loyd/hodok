use std::convert::From;
use std::io;
use std::result;

use super::i2c::I2C;


pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Unidentified
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

pub struct Hmc5883l {
    underline: I2C,
    running: bool,
    gain: f32,
    buf: [u8; 6]
}

impl Hmc5883l {
    pub fn new(bus: &str) -> Result<Hmc5883l> {
        Hmc5883l::with_addr(bus, 0x1e)
    }

    pub fn with_addr(bus: &str, addr: u16) -> Result<Hmc5883l> {
        let underline = try!(I2C::open(bus, addr));
        try!(Hmc5883l::identify(&underline));
        Ok(Hmc5883l {
            underline: underline,
            running: false,
            gain: 0.,
            buf: [0; 6]
        })
    }

    fn identify(i2c: &I2C) -> Result<()> {
        let mut check = [0, 0, 0];
        try!(i2c.read(0x0a, &mut check));
        if &check != b"H43" { Err(Error::Unidentified) } else { Ok(()) }
    }

    pub fn set_rate(&mut self, expected: f32) -> Result<f32> {
        static RATES: [f32; 7] = [0.75, 1.5, 3., 7.5, 15., 30., 75.];

        let ctl = RATES.iter().position(|x| expected <= *x).unwrap_or(6);
        let actual = RATES[ctl];

        self.buf[0] = 0x00;
        self.buf[1] = (ctl as u8) << 2;

        try!(self.underline.write(&self.buf[0..2]));
        Ok(actual)
    }

    pub fn set_range(&mut self, expected: f32) -> Result<f32> {
        static RANGES: [f32; 8] = [0.88, 1.3, 1.9, 2.5, 4., 4.7, 5.6, 8.1];

        let ctl = RANGES.iter().position(|x| expected <= *x).unwrap_or(7);
        let actual = RANGES[ctl];

        self.buf[0] = 0x01;
        self.buf[1] = (ctl as u8) << 5;

        self.gain = actual/2048. + 0.0003;

        try!(self.underline.write(&self.buf[0..2]));
        Ok(actual)
    }

    pub fn start(&mut self) -> Result<()> {
        self.buf[0] = 0x02;
        self.buf[1] = 0x00;
        try!(self.underline.write(&self.buf[0..2]));
        self.running = true;
        Ok(())
    }

    pub fn measure(&mut self) -> Result<(f32, f32, f32)> {
        try!(self.underline.read(0x03, &mut self.buf));
        Ok((
            ((self.buf[1] as i16) << 8 | (self.buf[0] as i16)) as f32 * self.gain,
            ((self.buf[3] as i16) << 8 | (self.buf[2] as i16)) as f32 * self.gain,
            ((self.buf[5] as i16) << 8 | (self.buf[4] as i16)) as f32 * self.gain
        ))
    }

    pub fn stop(&mut self) -> Result<()> {
        self.buf[0] = 0x02;
        self.buf[1] = 0x02;
        try!(self.underline.write(&self.buf[0..2]));
        self.running = false;
        Ok(())
    }
}

impl Drop for Hmc5883l {
    fn drop(&mut self) {
        if self.running {
            let _ = self.stop();
        }
    }
}
