use std::mem;

use constants;
use math::Madgwick;
use messages::Attitude;
use nodes::{Node, Output, timer};
use periphery::{Adxl345, Hmc5883l, L3g4200d};


pub struct Ahrs {
    pub attitude: Output<Attitude>
}

impl Node for Ahrs {
    fn new() -> Ahrs {
        Ahrs {
            attitude: Output::new()
        }
    }

    fn main(&mut self) {
        let rate = constants::AHRS_RATE;

        let mut accel = Adxl345::new(constants::AHRS_DEVICE).unwrap();
        accel.set_rate(rate).unwrap();
        accel.set_range(constants::ACCEL_RANGE).unwrap();

        let mut magn = Hmc5883l::new(constants::AHRS_DEVICE).unwrap();
        magn.set_rate(rate).unwrap();
        magn.set_range(constants::MAGN_RANGE).unwrap();

        let mut gyro = L3g4200d::new(constants::AHRS_DEVICE).unwrap();
        gyro.set_rate(rate).unwrap();
        gyro.set_range(constants::GYRO_RANGE).unwrap();

        let mut filter = Madgwick::new();

        accel.start().unwrap();
        magn.start().unwrap();

        for _ in timer(rate).iter() {
            let (gx, gy, gz) = gyro.measure().unwrap();

            let g = (gx.to_radians(), gy.to_radians(), gz.to_radians());
            let a = accel.measure().unwrap();
            let m = magn.measure().unwrap();

            let q = filter.update(g, a, m, rate.recip());

            self.attitude.send(unsafe { mem::transmute(q) });
        }
    }
}
