use std::f32::consts::PI;
use std::mem;

use base::node;
use constants::{AHRS_DEVICE, AHRS_RATE, ACCEL_RANGE, MAGN_RANGE, GYRO_RANGE};
use devices::Adxl345;
use devices::Hmc5883l;
use devices::L3g4200d;
use messages::Attitude;

use self::madgwick::Madgwick;

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
