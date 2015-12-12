use base::Result;
use ifaces::I2C;


pub struct L3g4200d {
    underline: I2C,
    running: bool,
    gain: f32,
    buf: [u8; 6]
}

impl L3g4200d {
    pub fn new(bus: &str) -> Result<L3g4200d> {
        L3g4200d::with_addr(bus, 0x69)
    }

    pub fn with_addr(bus: &str, addr: u16) -> Result<L3g4200d> {
        let underline = try!(I2C::open(bus, addr));
        try!(L3g4200d::identify(&underline));
        Ok(L3g4200d {
            underline: underline,
            running: false,
            gain: 0.,
            buf: [0; 6]
        })
    }

    fn identify(i2c: &I2C) -> Result<()> {
        let mut check = [0];
        try!(i2c.read(0x0f, &mut check));
        if check[0] != 0xd3 { Err(From::from("Unidentified device")) } else { Ok(()) }
    }

    pub fn set_rate(&mut self, expected: f32) -> Result<f32> {
        static RATES: [(f32, u8); 4] = [(100., 0x2f), (200., 0x6f),
                                        (400., 0xaf), (800., 0xef)];

        let pos = RATES.iter().position(|t| expected <= (*t).0).unwrap_or(3);
        let (actual, ctl) = RATES[pos];

        self.buf[0] = 0x20;
        self.buf[1] = ctl;

        try!(self.underline.write(&self.buf[0..2]));
        Ok(actual)
    }

    pub fn set_range(&mut self, expected: f32) -> Result<f32> {
        static RANGES: [(f32, u8); 3] = [(250., 0x00), (500., 0x10), (2000., 0x20)];

        let pos = RANGES.iter().position(|t| expected <= (*t).0).unwrap_or(2);
        let (actual, ctl) = RANGES[pos];

        self.buf[0] = 0x23;
        self.buf[1] = ctl;

        self.gain = actual/32768.;

        try!(self.underline.write(&self.buf[0..2]));
        Ok(actual)
    }

    pub fn measure(&mut self) -> Result<(f32, f32, f32)> {
        try!(self.underline.read(0x80 | 0x28, &mut self.buf));
        Ok((
            ((self.buf[1] as i16) << 8 | (self.buf[0] as i16)) as f32 * self.gain,
            ((self.buf[3] as i16) << 8 | (self.buf[2] as i16)) as f32 * self.gain,
            ((self.buf[5] as i16) << 8 | (self.buf[4] as i16)) as f32 * self.gain
        ))
    }

    pub fn stop(&mut self) -> Result<()> {
        self.buf[0] = 0x20;
        self.buf[1] = 0x00;
        try!(self.underline.write(&self.buf[0..2]));
        self.running = false;
        Ok(())
    }
}

impl Drop for L3g4200d {
    fn drop(&mut self) {
        if self.running {
            let _ = self.stop();
        }
    }
}
