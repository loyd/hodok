use std::io::Result;

use super::serial::Serial;


pub struct Maestro(Serial);

impl Maestro {
    pub fn new(device: &str) -> Result<Maestro> {
        Ok(Maestro(try!(Serial::open(device))))
    }

    pub fn set_target(&self, channel: u8, mut us: u16) -> Result<()> {
        us <<= 2;
        let (lb, mb) = ((us & 0x7f) as u8, ((us >> 7) & 0x7f) as u8);
        Ok(try!(self.0.write(&[0x84, channel, lb, mb])))
    }
}
