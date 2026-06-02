use std::f64::consts::PI;

use nalgebra::{Vector3, Vector6};

// ------- get_elements --------
// Inputs:
//         r: position vector at time t [m]
//         v: velocity vector at time t [m/s]
// Outputs:
//         (p): period [s]
//         (elements): 6 orbital elements [h, i, raan, e, w, theta]
//         (t_1): time since pariapsis
//         (r_p): Periapsis [m]
//         (r_a): Apoapsis [m]
pub fn get_elements(
    r: nalgebra::Vector3<f64>,
    v: nalgebra::Vector3<f64>,
    mu: f64,
) -> (f64, Vector6<f64>, f64, f64) {
    // Distance (r)
    let mag_r = r.magnitude();

    // Speed (v)
    let mag_v = v.magnitude();

    // Radial Velocity (v_r)
    let vr = r.dot(&v) / mag_r;

    /*
    match vr {
        vr if vr > 0. => println!("-> Object is flying away from periapsis"),
        vr if vr < 0. => println!("-> Object is flying towards periapsis"),
        _ => println!("vr = 0"),
    }
    */

    // Specific Angular Momentum (h):
    let h = r.cross(&v);

    // Magnitude of h                                       =>> 1st Element
    let mag_h = h.magnitude();

    // Inclination (i)                                      =>> 2nd Element
    let i = (h.z / mag_h).acos();

    // Node line Vector (N)
    let k: Vector3<f64> = Vector3::new(0., 0., 1.); // z-axis (K-hat) unit vector
    let n = k.cross(&h);

    // Magnitude of N
    let mag_n = n.magnitude();

    // Right Ascension of the Ascending Node (Omega) (RAAN) =>> 3rd Element
    let mut raan = (n.x / mag_n).acos();

    if n.y < 0. {
        raan = 360. * PI / 180. - (n.x / mag_n).acos();
    }

    // Eccentricity Vecotor (e)
    let e = (1. / mu) * ((mag_v.powi(2) - (mu / mag_r)) * &r - (mag_r * vr * &v));

    // Eccentricity                                         =>> 4th Element
    let mag_e = e.magnitude();

    // Argument of periapsis                                =>> 5th Element
    let mut w = (n.dot(&e) / (mag_n * mag_e)).acos();

    if e.z < 0. {
        w = 360. * PI / 180. - (n.dot(&e) / (mag_n * mag_e)).acos();
    }

    // True Anomaly:                                        =>>6th Element
    let mut theta = (e.dot(&r) / (mag_e * mag_r)).acos();

    if vr < 0. {
        theta = 360. * PI / 180. - (e.dot(&r) / (mag_e * mag_r)).acos();
    }

    // Perigee and Apogee Radii
    let r_p = (mag_h.powi(2) / mu) * 1. / (1. + mag_e * 0_f64.cos());
    let r_a = (mag_h.powi(2) / mu) * 1. / (1. + mag_e * 180_f64.to_radians().cos());

    // Semimajor Axis
    let a = 0.5 * (r_p + r_a);

    // Period
    let period = 2. * PI / mu.sqrt() * a.powf(1.5);

    // Eccentric Anomaly
    //let e1 = 2. * (((1. - mag_e) / (1. + mag_e)).sqrt() * (theta / 2.).tan()).atan();

    // Mean Anomaly
    //let me1 = e1 - (mag_e * e1.sin());

    // Time since periapsis
    //let t_1 = (mag_h.powi(3) / mu.powi(2)) * 1. / (1. - mag_e.powi(2)).powf(1.5) * me1;

    // Return a tuple of a vector containing elements + some extra info if necessary
    (
        period,
        Vector6::new(mag_h, i, raan, mag_e, w, theta),
        r_p,
        r_a,
    )
}
