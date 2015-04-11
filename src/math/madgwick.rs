pub struct Madgwick {
    attitude: (f32, f32, f32, f32),
    beta: f32
}

impl Madgwick {
    pub fn new() -> Madgwick {
        Madgwick::with_beta(0.1)
    }

    pub fn with_beta(beta: f32) -> Madgwick {
        assert!(beta > 0.);
        Madgwick {
            attitude: (1., 0., 0., 0.),
            beta: beta
        }
    }

    pub fn update(&mut self,
                  (gx, gy, gz): (f32, f32, f32),                // [rad/s]
                  (mut ax, mut ay, mut az): (f32, f32, f32),    // [g]
                  (mut mx, mut my, mut mz): (f32, f32, f32),    // [T] or [G]
                  dt: f32) -> (f32, f32, f32, f32) {            // [s]
        let mut q = self.attitude;

        let mut recip_norm;

        // Rate of change of quaternion from gyroscope.
        let mut qdot = (0.5 * (-q.1 * gx - q.2 * gy - q.3 * gz),
                        0.5 * (q.0 * gx + q.2 * gz - q.3 * gy),
                        0.5 * (q.0 * gy - q.1 * gz + q.3 * gx),
                        0.5 * (q.0 * gz + q.1 * gy - q.2 * gx));

        // Compute feedback only if accelerometer measurement valid.
        if !((ax == 0.) && (ay == 0.) && (az == 0.)) {
            // Normalize accelerometer measurement.
            recip_norm = (ax*ax + ay*ay + az*az).rsqrt();
            ax *= recip_norm;
            ay *= recip_norm;
            az *= recip_norm;

            // Normalize magnetometer measurement.
            recip_norm = (mx*mx + my*my + mz*mz).rsqrt();
            mx *= recip_norm;
            my *= recip_norm;
            mz *= recip_norm;

            // Auxiliary variables to avoid repeated arithmetic.
            let _2q0mx = 2. * q.0 * mx;
            let _2q0my = 2. * q.0 * my;
            let _2q0mz = 2. * q.0 * mz;
            let _2q1mx = 2. * q.1 * mx;
            let _2q0 = 2. * q.0;
            let _2q1 = 2. * q.1;
            let _2q2 = 2. * q.2;
            let _2q3 = 2. * q.3;
            let _2q0q2 = 2. * q.0 * q.2;
            let _2q2q3 = 2. * q.2 * q.3;
            let q0q0 = q.0 * q.0;
            let q0q1 = q.0 * q.1;
            let q0q2 = q.0 * q.2;
            let q0q3 = q.0 * q.3;
            let q1q1 = q.1 * q.1;
            let q1q2 = q.1 * q.2;
            let q1q3 = q.1 * q.3;
            let q2q2 = q.2 * q.2;
            let q2q3 = q.2 * q.3;
            let q3q3 = q.3 * q.3;

            // Reference direction of Earth's magnetic field.
            let hx = mx * q0q0 - _2q0my * q.3 + _2q0mz * q.2 + mx * q1q1 + _2q1 * my * q.2
                       + _2q1 * mz * q.3 - mx * q2q2 - mx * q3q3;
            let hy = _2q0mx * q.3 + my * q0q0 - _2q0mz * q.1 + _2q1mx * q.2 - my * q1q1
                       + my * q2q2 + _2q2 * mz * q.3 - my * q3q3;
            let _2bx = (hx * hx + hy * hy).sqrt();
            let _2bz = -_2q0mx * q.2 + _2q0my * q.1 + mz * q0q0 + _2q1mx * q.3 - mz * q1q1
                 + _2q2 * my * q.3 - mz * q2q2 + mz * q3q3;
            let _4bx = 2. * _2bx;
            let _4bz = 2. * _2bz;

            // Gradient descent algorithm corrective step.
            let s = (-_2q2 * (2.*q1q3 - _2q0q2 - ax) + _2q1 * (2.*q0q1 + _2q2q3 - ay)
                     - _2bz * q.2 * (_2bx * (0.5 - q2q2 - q3q3) + _2bz * (q1q3 - q0q2) - mx)
                     + (-_2bx * q.3 + _2bz * q.1) * (_2bx * (q1q2 - q0q3) + _2bz * (q0q1 + q2q3)
                     - my) + _2bx * q.2 * (_2bx *(q0q2+q1q3) + _2bz*(0.5 - q1q1 - q2q2) - mz),

                     _2q3 * (2.*q1q3 - _2q0q2 - ax) + _2q0 * (2.*q0q1 + _2q2q3 - ay)
                     - 4. * q.1 * (1. - 2.*q1q1 - 2.*q2q2 - az) + _2bz * q.3 * (_2bx
                     * (0.5 - q2q2 - q3q3) + _2bz * (q1q3 - q0q2) - mx) + (_2bx * q.2 + _2bz
                     * q.0) * (_2bx * (q1q2 - q0q3) + _2bz * (q0q1 + q2q3) - my) + (_2bx * q.3
                     - _4bz * q.1) * (_2bx * (q0q2 + q1q3) + _2bz * (0.5 - q1q1 - q2q2) - mz),

                     -_2q0 * (2.*q1q3 - _2q0q2 - ax) + _2q3 * (2.*q0q1 + _2q2q3 - ay)
                     - 4. * q.2 * (1. - 2.*q1q1 - 2.*q2q2 - az) + (-_4bx * q.2 - _2bz * q.0)
                     * (_2bx * (0.5 - q2q2 - q3q3) + _2bz * (q1q3 - q0q2) - mx) + (_2bx * q.1
                     + _2bz * q.3) * (_2bx * (q1q2 - q0q3) + _2bz * (q0q1 + q2q3) - my) + (_2bx
                     * q.0 - _4bz*q.2) * (_2bx * (q0q2+q1q3) + _2bz * (0.5 - q1q1-q2q2) - mz),

                     _2q1 * (2.*q1q3 - _2q0q2 - ax) + _2q2 * (2.*q0q1 + _2q2q3 - ay)
                     + (-_4bx * q.3 + _2bz * q.1) * (_2bx * (0.5 - q2q2 - q3q3) + _2bz * (q1q3
                     - q0q2) - mx) + (-_2bx * q.0 + _2bz * q.2) * (_2bx * (q1q2 - q0q3) + _2bz
                     * (q0q1 + q2q3) - my) + _2bx * q.1 * (_2bx * (q0q2 + q1q3) + _2bz * (0.5
                     - q1q1 - q2q2) - mz));

            recip_norm = (s.0*s.0 + s.1*s.1 + s.2*s.2 + s.3*s.3).rsqrt();

            // Apply feedback step.
            qdot.0 -= self.beta * s.0 * recip_norm;
            qdot.1 -= self.beta * s.1 * recip_norm;
            qdot.2 -= self.beta * s.2 * recip_norm;
            qdot.3 -= self.beta * s.3 * recip_norm;
        }

        // Integrate rate of change of quaternion to yield quaternion.
        q.0 = qdot.0.mul_add(dt, q.0);
        q.1 = qdot.1.mul_add(dt, q.1);
        q.2 = qdot.2.mul_add(dt, q.2);
        q.3 = qdot.3.mul_add(dt, q.3);

        // Normalize quaternion.
        recip_norm = (q.0*q.0 + q.1*q.1 + q.2*q.2 + q.3*q.3).rsqrt();
        self.attitude.0 = q.0 * recip_norm;
        self.attitude.1 = q.1 * recip_norm;
        self.attitude.2 = q.2 * recip_norm;
        self.attitude.3 = q.3 * recip_norm;

        self.attitude
    }
}
