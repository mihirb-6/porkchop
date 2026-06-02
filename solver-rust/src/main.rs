use csv::Writer;
#[allow(unused)]
use inquire::Select;
use lambert_izzo::{lambert, LambertInput, RevolutionBudget, TransferWay};
use satkit::consts::MU_SUN;
use satkit::jplephem::barycentric_state;
#[allow(unused)]
use satkit::prelude::*;
use satkit::{Duration, Instant};
#[allow(unused)]
use std::collections::HashMap;
use std::{error::Error, f64::consts::PI};

//mod elements;

// (start, stop, step size)
struct StepRange(f64, f64, f64);

impl Iterator for StepRange {
    type Item = f64;

    #[inline]
    fn next(&mut self) -> Option<f64> {
        if self.0 < self.1 {
            let v = self.0;
            self.0 = v + self.2;
            Some(v)
        } else {
            None
        }
    }
}

fn find_trajectories(
    departure_obj: satkit::SolarSystem,
    arrival_obj: satkit::SolarSystem,
    min_departure_days: i32,
    max_departure_days: i32,
    shortest_arrival_tof: i32,
    longest_arrival_tof: i32,
    dep_step_size: f64,
) -> (
    Vec<(Instant, Instant, f64, f64)>,
    Vec<(Instant, Instant, f64, f64)>,
) {
    println!("Calculating trajectories...");
    // dep_date, arr_date, dep_c3, arr_c3,
    let mut type1_data: Vec<(Instant, Instant, f64, f64)> = Vec::new();
    let mut type2_data: Vec<(Instant, Instant, f64, f64)> = Vec::new();

    let now = satkit::Instant::now();

    for dep_day in StepRange(
        min_departure_days as f64,
        max_departure_days as f64,
        dep_step_size,
    ) {
        for tof in StepRange(shortest_arrival_tof as f64, longest_arrival_tof as f64, 1.) {
            let departure_date = now + satkit::Duration::from_days(dep_day as f64);
            let arrival_date = departure_date + Duration::from_days(tof as f64);

            let (p1_pos, p1_vel) = barycentric_state(departure_obj, &departure_date).unwrap();
            let (p2_pos, p2_vel) = barycentric_state(arrival_obj, &arrival_date).unwrap();

            let p1_pos: [f64; 3] = [p1_pos[0] / 1e3, p1_pos[1] / 1e3, p1_pos[2] / 1e3];
            let p2_pos: [f64; 3] = [p2_pos[0] / 1e3, p2_pos[1] / 1e3, p2_pos[2] / 1e3];
            let p1_vel: [f64; 3] = [p1_vel[0] / 1e3, p1_vel[1] / 1e3, p1_vel[2] / 1e3];
            let p2_vel: [f64; 3] = [p2_vel[0] / 1e3, p2_vel[1] / 1e3, p2_vel[2] / 1e3];

            let tof_s = tof as f64 * 86400.;

            let short_input = LambertInput {
                r1: p1_pos,
                r2: p2_pos,
                tof: tof_s,
                mu: MU_SUN / 1e9,
                way: TransferWay::Short,
                revolutions: RevolutionBudget::SingleOnly,
            };
            let long_input = LambertInput {
                r1: p1_pos,
                r2: p2_pos,
                tof: tof_s,
                mu: MU_SUN / 1e9,
                way: TransferWay::Long,
                revolutions: RevolutionBudget::SingleOnly,
            };
            let short = lambert(&short_input).unwrap();
            let long = lambert(&long_input).unwrap();

            let v1_short = short.single.v1;
            let v2_short = short.single.v2;

            let v1_long = long.single.v1;
            let v2_long = long.single.v2;

            // DEPARTURE EXCESS VELOCITIES
            let dep_vinf_type1: [f64; 3] = [
                v1_short[0] - p1_vel[0],
                v1_short[1] - p1_vel[1],
                v1_short[2] - p1_vel[2],
            ];
            let dep_vinf_type2: [f64; 3] = [
                v1_long[0] - p1_vel[0],
                v1_long[1] - p1_vel[1],
                v1_long[2] - p1_vel[2],
            ];

            // ARRIVAL EXCESS VELOCITIES
            let arr_vinf_type1: [f64; 3] = [
                v2_short[0] - p2_vel[0],
                v2_short[1] - p2_vel[1],
                v2_short[2] - p2_vel[2],
            ];
            let arr_vinf_type2: [f64; 3] = [
                v2_long[0] - p2_vel[0],
                v2_long[1] - p2_vel[1],
                v2_long[2] - p2_vel[2],
            ];

            // DEPARTURE C3 ENERGIES
            let dep_c3_type1 = magnitude(&dep_vinf_type1).powi(2);
            let dep_c3_type2 = magnitude(&dep_vinf_type2).powi(2);

            // ARRIVAL C3 ENERGIES
            let arr_c3_type1 = magnitude(&arr_vinf_type1).powi(2);
            let arr_c3_type2 = magnitude(&arr_vinf_type2).powi(2);

            type1_data.push((departure_date, arrival_date, dep_c3_type1, arr_c3_type1));

            type2_data.push((departure_date, arrival_date, dep_c3_type2, arr_c3_type2));
        }
    }

    println!(
        "Found {} Type 1 and {} Type 2 Trajectories",
        type1_data.len(),
        type2_data.len()
    );

    (type1_data, type2_data)
}

fn main() -> Result<(), Box<dyn Error>> {
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
    let (type1_data, type2_data) =
        find_trajectories(SolarSystem::EMB, SolarSystem::Mars, 60, 300, 120, 500, 1.);

    // ===================
    // PROGRADE CSV OUTPUT
    // ===================

    println!("Writing Type I Data to CSV...");
    let mut wtr = Writer::from_path("/Users/memes/projects/porkchop_plot/TYPEI_OUTPUT.csv")?;

    wtr.write_record(&[
        "Departure Date [JD]",
        "Arrival Date [JD]",
        "Departure C3 [km^2/s^2]",
        "Arrival C3 [km^2/s^2]",
    ])
    .expect("Failed to write headers");

    for (dep_date, arr_date, dep_c3, arr_c3) in &type1_data {
        wtr.write_record(&[
            dep_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            arr_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            dep_c3.to_string(),
            arr_c3.to_string(),
        ])
        .expect("Failed to write record")
    }

    wtr.flush()?;
    println!("Wrote to CSV.");

    // ===================
    // RETROGRADE CSV OUTPUT
    // ===================

    println!("Writing Type II Data to CSV...");
    let mut wtr = Writer::from_path("/Users/memes/projects/porkchop_plot/TYPEII_OUTPUT.csv")?;

    wtr.write_record(&[
        "Departure Date [JD]",
        "Arrival Date [JD]",
        "Departure C3 [km^2/s^2]",
        "Arrival C3 [km^2/s^2]",
    ])
    .expect("Failed to write headers");

    for (dep_date, arr_date, dep_c3, arr_c3) in &type2_data {
        wtr.write_record(&[
            dep_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            arr_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            dep_c3.to_string(),
            arr_c3.to_string(),
        ])
        .expect("Failed to write record")
    }

    wtr.flush()?;
    println!("Wrote to CSV.");

    #[allow(unused)]
    //let test_time = find_zero_phase(SolarSystem::EMB, SolarSystem::Mars, 1.0, 0.25);
    Ok(())
}

fn magnitude(matrix: &[f64; 3]) -> f64 {
    (matrix[0].powi(2) + matrix[1].powi(2) + matrix[2].powi(2)).sqrt()
}

#[allow(unused)]
//tau_s = 2pi / |w_p1 - w_p2| -> w is rotational rates about the sun = 2pi / |1/p1_period - 1/p2_period|
fn synodic_period(planet1: SolarSystem, planet2: SolarSystem) -> f64 {
    let time = Instant::now();
    let (position1, velocity1) = satkit::jplephem::barycentric_state(planet1, &time).unwrap();
    let (position2, velocity2) = satkit::jplephem::barycentric_state(planet2, &time).unwrap();
    let period1 = satkit::Kepler::from_pv(position1, velocity1)
        .unwrap()
        .period();
    let period2 = satkit::Kepler::from_pv(position2, velocity2)
        .unwrap()
        .period();

    2. * PI / (period1 - period2).abs()
}

/*
 * Random Functions
// calculates straight line distance between two position vectors
fn get_distance(planet1: Matrix<f64, 3, 1>, planet2: Matrix<f64, 3, 1>) -> f64 {
    ((planet2[0] - planet1[0]).powi(2)
        + (planet2[1] - planet1[1]).powi(2)
        + (planet2[2] - planet1[2]).powi(2))
    .sqrt()
}
// calculates the phase angle between two position vectors
fn calculate_phase_angle(planet1: &Matrix<f64, 3, 1>, planet2: &Matrix<f64, 3, 1>) -> f64 {
    let lambda1 = planet1[1].atan2(planet1[0]);
    let lambda2 = planet2[1].atan2(planet2[0]);

    ((lambda1 - lambda2).abs() % (2. * std::f64::consts::PI)).to_degrees()
}
*/

/*
// Add to the beginning of main to select objects from terminal
let sol_sys_objs = vec![
    "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
];

let departure_object = Select::new("Select departure body:", sol_sys_objs.clone())
    .prompt()
    .expect("Failed to select an object.");

let arrival_choices: Vec<&str> = sol_sys_objs
    .into_iter()
    .filter(|&obj| obj != departure_object)
    .collect();

let arrival_object = Select::new("Select arrival body:", arrival_choices)
    .prompt()
    .expect("Failed to select an object.");

// Dictionary for satkit SolarSystem Enums
let solsystem_bodies: HashMap<&str, satkit::SolarSystem> = HashMap::from([
    ("Mercury", SolarSystem::Mercury),
    ("Venus", SolarSystem::Venus),
    ("Earth", SolarSystem::EMB),
    ("Mars", SolarSystem::Mars),
    ("Jupiter", SolarSystem::Jupiter),
    ("Saturn", SolarSystem::Saturn),
    ("Uranus", SolarSystem::Uranus),
    ("Neptune", SolarSystem::Neptune),
]);
*/
//input if selecting in terminal: solsystem_bodies[&departure_object], solsystem_bodies[&arrival_object],
