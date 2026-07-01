use lambert_izzo::{LambertInput, RevolutionBudget, TransferWay, lambert};
use satkit::consts::MU_SUN;
use satkit::jplephem::barycentric_state;
use satkit::{Duration, Instant};

// A range struct similar to np.arange(): (start, stop, step size)
struct StepRange(f64, f64, f64);

// Implementing an Iterator for StepRange
impl Iterator for StepRange {
    type Item = f64;

    #[inline]
    fn next(&mut self) -> Option<f64> {
        // self.0 = starting value
        // self.1 = ending value
        // self.2 = step size value

        if self.0 < self.1 {
            let v = self.0;
            self.0 = v + self.2;
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchBounds {
    pub dep_start: i32,
    pub dep_end: i32,
    pub tof_min: i32,
    pub tof_max: i32,
    pub step_size: f64,
}

/* Find Trajectories
 * Inputs:
 *      departure_obj
 *      arrival_obj
 *      min_departure_days
 *      max_departure_days
 *      shortest_arrival_tof
 *      longest_arrival_tof
 *      dep_step_size
 * Outputs:
 *      Two Vectors of type: Vec<(Instant, Instant, f64, f64)>
 * */

pub fn find_trajectories(
    initial_time: Instant,
    departure_obj: satkit::SolarSystem,
    arrival_obj: satkit::SolarSystem,
    search: SearchBounds,
) -> (
    Vec<(Instant, Instant, f64, f64, f64, f64)>,
    Vec<(Instant, Instant, f64, f64, f64, f64)>,
) {
    println!("===========================================================");
    println!("Calculating trajectories...");
    // dep_date, arr_date, dep_c3, arr_c3,
    let mut type1_data: Vec<(Instant, Instant, f64, f64, f64, f64)> = Vec::new();
    let mut type2_data: Vec<(Instant, Instant, f64, f64, f64, f64)> = Vec::new();

    for dep_day in StepRange(
        search.dep_start as f64,
        search.dep_end as f64,
        search.step_size,
    ) {
        for tof in StepRange(
            search.tof_min as f64,
            search.tof_max as f64,
            search.step_size,
        ) {
            let departure_date = initial_time + satkit::Duration::from_days(dep_day as f64);
            let arrival_date = departure_date + Duration::from_days(tof as f64);

            let (r1, v1) = barycentric_state(departure_obj, &departure_date).unwrap();
            let (r2, v2) = barycentric_state(arrival_obj, &arrival_date).unwrap();

            let r1: [f64; 3] = [r1[0] / 1e3, r1[1] / 1e3, r1[2] / 1e3];
            let r2: [f64; 3] = [r2[0] / 1e3, r2[1] / 1e3, r2[2] / 1e3];
            let v1: [f64; 3] = [v1[0] / 1e3, v1[1] / 1e3, v1[2] / 1e3];
            let v2: [f64; 3] = [v2[0] / 1e3, v2[1] / 1e3, v2[2] / 1e3];

            let tof_s = tof as f64 * 86400.;

            // CONSTRUCT TYPE I LAMBERT SOLVER INPUT
            let short_input = LambertInput {
                r1: r1,
                r2: r2,
                tof: tof_s,
                mu: MU_SUN / 1e9,
                way: TransferWay::Short,
                revolutions: RevolutionBudget::SingleOnly,
            };
            // CONSTRUCT TYPE II LAMBERT SOLVER INPUT
            let long_input = LambertInput {
                r1: r1,
                r2: r2,
                tof: tof_s,
                mu: MU_SUN / 1e9,
                way: TransferWay::Long,
                revolutions: RevolutionBudget::SingleOnly,
            };

            // Find solution to Lambert's Problem using LambertInput
            let short = lambert(&short_input).unwrap();
            let long = lambert(&long_input).unwrap();

            // Not necessary, just for readability
            let v1_short = short.single.v1;
            let v2_short = short.single.v2;

            let v1_long = long.single.v1;
            let v2_long = long.single.v2;

            // TYPE I DEPARTURE EXCESS VELOCITY
            let dep_vinf_type1: [f64; 3] = [
                v1_short[0] - v1[0],
                v1_short[1] - v1[1],
                v1_short[2] - v1[2],
            ];
            // TYPE II DEPARTURE EXCESS VELOCITY
            let dep_vinf_type2: [f64; 3] =
                [v1_long[0] - v1[0], v1_long[1] - v1[1], v1_long[2] - v1[2]];

            // TYPE I ARRIVAL EXCESS VELOCITY
            let arr_vinf_type1: [f64; 3] = [
                v2_short[0] - v2[0],
                v2_short[1] - v2[1],
                v2_short[2] - v2[2],
            ];
            // TYPE II ARRIVAL EXCESS VELOCITY
            let arr_vinf_type2: [f64; 3] =
                [v2_long[0] - v2[0], v2_long[1] - v2[1], v2_long[2] - v2[2]];

            // TYPE I DECLINATION OF LAUNCH ASYMPTOTE
            let dla_t1 = ((dep_vinf_type1[2] / magnitude(&dep_vinf_type1)).asin()).to_degrees();

            //TYPE II DECLINATION OF LAUNCH ASYMPTOTE
            let dla_t2 = ((dep_vinf_type2[2] / magnitude(&dep_vinf_type2)).asin()).to_degrees();

            // TYPE I RIGHT ASCENSION OF LAUNCH ASYMPTOTE
            let mut rla_t1 = (dep_vinf_type1[1].atan2(dep_vinf_type1[0])).to_degrees();
            if rla_t1 < 0. {
                rla_t1 += 360.;
            }
            // TYPE II RIGHT ASCENSION OF LAUNCH ASYMPTOTE
            let mut rla_t2 = (dep_vinf_type2[1].atan2(dep_vinf_type2[0])).to_degrees();
            if rla_t2 < 0. {
                rla_t2 += 360.;
            }
            // DEPARTURE C3 ENERGIES
            let dep_c3_type1 = magnitude(&dep_vinf_type1).powi(2);
            let dep_c3_type2 = magnitude(&dep_vinf_type2).powi(2);

            // ARRIVAL C3 ENERGIES
            let arr_c3_type1 = magnitude(&arr_vinf_type1).powi(2);
            let arr_c3_type2 = magnitude(&arr_vinf_type2).powi(2);

            // Push to respective vectors
            type1_data.push((
                departure_date,
                arrival_date,
                dep_c3_type1,
                arr_c3_type1,
                dla_t1,
                rla_t1,
            ));
            type2_data.push((
                departure_date,
                arrival_date,
                dep_c3_type2,
                arr_c3_type2,
                dla_t2,
                rla_t2,
            ));
        }
    }

    // Print the number of trajectories (data points) found
    println!(
        "Found {} Type 1 and {} Type 2 Trajectories",
        type1_data.len(),
        type2_data.len()
    );
    println!("===========================================================");

    (type1_data, type2_data)
}

/* Simple Magnitude Calculation */
fn magnitude(matrix: &[f64; 3]) -> f64 {
    (matrix[0].powi(2) + matrix[1].powi(2) + matrix[2].powi(2)).sqrt()
}

/* Ecliptic to Equatorial Coordinate Frame Transformation
fn ecliptic_to_equatorial(ecliptic: &[f64; 3]) -> [f64; 3] {
    // earth's axial tilt = 23.44 deg
    let tilt = 23.44_f64.to_radians
    let m: [[f64; 3]; 3] = [
        [1., 0., 0.],
        [0., tilt.cos(), -(tilt.sin())],
        [0., tilt.sin(), -(tilt.cos())],
    ];
    let res: [f64; 3] = [
        m[0][0] * ecliptic[0] + m[0][1] * ecliptic[1] + m[0][2] * ecliptic[2],
        m[1][0] * ecliptic[0] + m[1][1] * ecliptic[1] + m[1][2] * ecliptic[2],
        m[2][0] * ecliptic[0] + m[2][1] * ecliptic[1] + m[2][2] * ecliptic[2],
    ];

    res
}
*/
