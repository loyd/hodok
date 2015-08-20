use std::f32::consts::PI;
use std::mem;

use constants::{AHRS_DEVICE, AHRS_RATE, ACCEL_RANGE, MAGN_RANGE, GYRO_RANGE};
use node;

use messages::Attitude;

use self::adxl345::Adxl345;
use self::hmc5883l::Hmc5883l;
use self::l3g4200d::L3g4200d;
use self::madgwick::Madgwick;

mod adxl345;
mod hmc5883l;
mod i2c;
mod l3g4200d;
mod madgwick;


pub fn worker() {
    let attitude_tx = node::advertise::<Attitude>();

    let mut accel = Adxl345::new(AHRS_DEVICE).unwrap();
    accel.set_rate(AHRS_RATE).unwrap();
    accel.set_range(ACCEL_RANGE).unwrap();

    let mut magn = Hmc5883l::new(AHRS_DEVICE).unwrap();
    magn.set_rate(AHRS_RATE).unwrap();
    magn.set_range(MAGN_RANGE).unwrap();

    let mut gyro = L3g4200d::new(AHRS_DEVICE).unwrap();
    gyro.set_rate(AHRS_RATE).unwrap();
    gyro.set_range(GYRO_RANGE).unwrap();

    let mut filter = Madgwick::new();

    accel.start().unwrap();
    magn.start().unwrap();

    for _ in node::periodic(AHRS_RATE) {
        let (gx, gy, gz) = gyro.measure().unwrap();

        const DEG_TO_RAD: f32 = PI / 180.;
        let g = (gx * DEG_TO_RAD, gy * DEG_TO_RAD, gz * DEG_TO_RAD);
        let a = accel.measure().unwrap();
        let m = magn.measure().unwrap();

        let q = filter.update(g, a, m, AHRS_RATE.recip());

        attitude_tx.send(unsafe { mem::transmute(q) });
    }
}
