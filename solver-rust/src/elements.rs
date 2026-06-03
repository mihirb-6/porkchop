#[allow(unused)]
use numeris::Vector3;
use satkit::consts::MU_SUN;
use std::f64::consts::PI;

#[allow(unused)]
#[derive(Debug)]
pub struct Elements {
    pub angular_momentum: f64,      // kg * m^2 / s
    pub inclination: f64,           //rad
    pub raan: f64,                  // rad
    pub eccentricity: f64,          // dimensionless
    pub argument_of_periapsis: f64, // rad
    pub true_anomaly: f64,          // rad
}

#[allow(unused)]
impl Elements {
    pub fn period(&self) -> f64 {
        let a = self.seminajor_axis();
        2. * PI * (a.powi(3) / MU_SUN).sqrt()
    }
    pub fn seminajor_axis(&self) -> f64 {
        let r_p = self.periapsis();
        let r_a = self.apoapsis();
        (r_p + r_a) / 2.
    }
    pub fn eccentric_anomaly(&self) -> f64 {
        let e = self.eccentricity;
        let theta = self.true_anomaly;
        2. * (((1. - e) / (1. + e)).sqrt() * (theta / 2.).tan()).atan()
    }
    pub fn mean_anomaly(&self) -> f64 {
        let e1 = self.eccentric_anomaly();
        let e = self.eccentricity;

        e1 - (e * e1.sin())
    }
    pub fn time_since_perapsis(&self) -> f64 {
        let h = self.angular_momentum;
        let e = self.eccentricity;
        let me1 = self.mean_anomaly();

        (h.powi(3) / MU_SUN.powi(2)) * 1. / (1. - e.powi(2)).powf(1.5) * me1
    }
    pub fn periapsis(&self) -> f64 {
        let h = self.angular_momentum;
        let e = self.eccentricity;
        (h.powi(2) / MU_SUN) * 1. / (1. + e * 0_f64.cos())
    }
    pub fn apoapsis(&self) -> f64 {
        let h = self.angular_momentum;
        let e = self.eccentricity;
        (h.powi(2) / MU_SUN) * 1. / (1. + e * 180_f64.to_radians().cos())
    }
}

// ------- get_elements --------
// Inputs:
//         r: position vector at time t [m]
//         v: velocity vector at time t [m/s]
// Outputs:
//         (elements): 6 orbital elements [h, i, raan, e, w, theta]
pub fn get_elements(r_vector: numeris::Vector3<f64>, v_vector: numeris::Vector3<f64>) -> Elements {
    // Distance (r)
    let r = r_vector.norm();

    // Speed (v)
    let v = v_vector.norm();

    // Radial Velocity (v_r)
    let v_r = v_vector.dot(&r_vector) / r;

    // Azimuthal Velocity (v_perp)
    #[allow(unused)]
    let v_perp = (v.powi(2) - v_r.powi(2)).sqrt();

    /*
    match vr {
        vr if vr > 0. => println!("-> Object is flying away from periapsis"),
        vr if vr < 0. => println!("-> Object is flying towards periapsis"),
        _ => println!("vr = 0"),
    }
    */

    // Specific Angular Momentum Vector (h):
    let h_vector = r_vector.cross(&v_vector);

    // Magnitude of h                                       =>> 1st Element
    let h = h_vector.norm();

    // Inclination (i)                                      =>> 2nd Element
    let i = (h_vector.z() / h).acos();

    // Node line Vector (N)
    // z-axis (K-hat) unit vector
    let k: numeris::Vector3<f64> = numeris::Vector3::from_array([0., 0., 1.]);
    let n_vector = k.cross(&h_vector);

    // Magnitude of N
    let n = n_vector.norm();

    // Right Ascension of the Ascending Node (Omega) (RAAN) =>> 3rd Element
    let mut raan = (n_vector.x() / n).acos();
    if n_vector.y() < 0. {
        raan = 2. * PI - (n_vector.x() / n).acos();
    }

    // Eccentricity Vecotor (e)
    let e_vector = (r_vector.cross(&h_vector) / MU_SUN) - (r_vector / r);

    // Eccentricity                                         =>> 4th Element
    let e = e_vector.norm();

    // Argument of periapsis                                =>> 5th Element
    let mut w = (e_vector.dot(&n_vector) / (e * n)).acos();

    if e_vector.z() < 0. {
        w = 2. * PI - (e_vector.dot(&n_vector) / (e * n)).acos();
    }

    // True Anomaly:                                        =>>6th Element
    let mut theta = (e_vector.dot(&r_vector) / (e * r)).acos();
    if v_r < 0. {
        theta = 2. * PI - (e_vector.dot(&r_vector) / (e * r)).acos();
    }

    let orbital_elements = Elements {
        angular_momentum: h,
        inclination: i,
        raan: raan,
        eccentricity: e,
        argument_of_periapsis: w,
        true_anomaly: theta,
    };
    // Return a tuple of a vector containing elements
    orbital_elements
}
