use csv::Writer;
use lambert_izzo::{LambertInput, RevolutionBudget, TransferWay, lambert};
use satkit::consts::MU_SUN;
use satkit::jplephem::barycentric_state;
use satkit::prelude::*;
use satkit::{Duration, Instant};
use std::error::Error;

mod elements;

use crate::elements::{find_periods, hohmann_transfer_time};

//mod elements;

// (start, stop, step size)
struct StepRange(f64, f64, f64);

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
struct SearchBounds {
    dep_start: i32,
    dep_end: i32,
    tof_min: i32,
    tof_max: i32,
    step_size: f64,
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
fn find_trajectories(
    initial_time: Instant,
    departure_obj: satkit::SolarSystem,
    arrival_obj: satkit::SolarSystem,
    search: SearchBounds,
) -> (
    Vec<(Instant, Instant, f64, f64)>,
    Vec<(Instant, Instant, f64, f64)>,
) {
    println!("===========================================================");
    println!("Calculating trajectories...");
    // dep_date, arr_date, dep_c3, arr_c3,
    let mut type1_data: Vec<(Instant, Instant, f64, f64)> = Vec::new();
    let mut type2_data: Vec<(Instant, Instant, f64, f64)> = Vec::new();

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

            let short_input = LambertInput {
                r1: r1,
                r2: r2,
                tof: tof_s,
                mu: MU_SUN / 1e9,
                way: TransferWay::Short,
                revolutions: RevolutionBudget::SingleOnly,
            };
            let long_input = LambertInput {
                r1: r1,
                r2: r2,
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
                v1_short[0] - v1[0],
                v1_short[1] - v1[1],
                v1_short[2] - v1[2],
            ];
            let dep_vinf_type2: [f64; 3] =
                [v1_long[0] - v1[0], v1_long[1] - v1[1], v1_long[2] - v1[2]];

            // ARRIVAL EXCESS VELOCITIES
            let arr_vinf_type1: [f64; 3] = [
                v2_short[0] - v2[0],
                v2_short[1] - v2[1],
                v2_short[2] - v2[2],
            ];
            let arr_vinf_type2: [f64; 3] =
                [v2_long[0] - v2[0], v2_long[1] - v2[1], v2_long[2] - v2[2]];

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
    println!("===========================================================");

    (type1_data, type2_data)
}

fn main() {
    // Define initial time
    let initial_time = satkit::Instant::now();

    // Define Departure and Arrival Locations
    let departure_object = SolarSystem::EMB;
    let arrival_object = SolarSystem::Mars;

    println!("===========================================================");
    println!(
        "Searching for trajectories from {} to {} after datetime: {}...",
        departure_object,
        arrival_object,
        initial_time.strftime("%B-%d-%Y %H:%M:%S").unwrap()
    );

    // Synodic Period
    let syn_p = find_periods(departure_object, arrival_object, initial_time).synodic_period;
    println!("Synodic Period: {:.2} days...", syn_p);

    let hohmann_t = hohmann_transfer_time(departure_object, arrival_object, initial_time);
    println!("Hohmann Transfer Time: {:.2} days...", hohmann_t);

    println!("===========================================================");

    let search = SearchBounds {
        dep_start: (0.1 * syn_p) as i32,
        dep_end: (0.5 * syn_p) as i32,
        tof_min: (0.1 * hohmann_t) as i32,
        tof_max: (2.5 * hohmann_t) as i32,
        step_size: 1.5,
    };

    let d_init = initial_time + satkit::Duration::from_days(search.dep_start as f64);
    let d_end = initial_time + satkit::Duration::from_days(search.dep_end as f64);

    println!(
        "Departure range {} - {}",
        d_init.strftime("%B-%d-%Y %H:%M:%S").unwrap(),
        d_end.strftime("%B-%d-%Y %H:%M:%S").unwrap(),
    );
    println!(
        "TOF range [{:.2} - {:.2}] days",
        search.tof_min, search.tof_max
    );
    println!("===========================================================");

    // Calculate All Trajectories and Compute Delta-V
    // dep_obj, arr_obj, min_dep, max_dep, min_tof, max_tof, dep_step_size
    let (mut type1_data, mut type2_data) =
        find_trajectories(initial_time, departure_object, arrival_object, search);

    // Replace last two elements in data vector tuples with clipped values
    let max_c3 = 100.0;
    type1_data = type1_data
        .iter()
        .map(|(dep_date, arr_date, dep_c3, arr_c3)| (*dep_date, *arr_date, (*dep_c3).clamp(0.0, max_c3), (*arr_c3).clamp(0.0, max_c3)))
        .collect();
    type2_data = type2_data
        .iter()
        .map(|(dep_date, arr_date, dep_c3, arr_c3)| (*dep_date, *arr_date, (*dep_c3).clamp(0.0, max_c3), (*arr_c3).clamp(0.0, max_c3)))
        .collect();

    // Write data to separate csv's
    let type1_path = "/Users/mihir/projects/porkchop/plotter-python/TYPEI_DATA.csv";
    let type2_path = "/Users/mihir/projects/porkchop/plotter-python/TYPEII_DATA.csv";
    let meta_path = "/Users/mihir/projects/porkchop/plotter-python/METADATA.csv";
    write_to_csv(type1_path, &type1_data).unwrap();
    write_to_csv(type2_path, &type2_data).unwrap();

    let mut metadata_vector: Vec<(Instant, SearchBounds, f64)> = Vec::new();
    metadata_vector.push((initial_time, search.clone(), max_c3));
    write_metadata_to_csv(meta_path, &metadata_vector).unwrap();
}

/* Helper Funcion to write trajectory data to a CSV */
fn write_to_csv(
    path: &'static str,
    data: &Vec<(Instant, Instant, f64, f64)>,
) -> Result<(), Box<dyn Error>> {
    println!("Writing {}...", path);

    let mut wtr = Writer::from_path(path)?;

    wtr.write_record(&[
        "Departure Date [JD]",
        "Arrival Date [JD]",
        "Departure C3 [km^2/s^2]",
        "Arrival C3 [km^2/s^2]",
    ])
    .expect("Failed to write headers");

    for (dep_date, arr_date, dep_c3, arr_c3) in data {
        wtr.write_record(&[
            dep_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            arr_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            dep_c3.to_string(),
            arr_c3.to_string(),
        ])
        .expect("Failed to write record")
    }

    wtr.flush()?;
    println!("Wrote data to CSV.");
    Ok(())
}

fn write_metadata_to_csv(
    path: &'static str,
    data: &Vec<(Instant, SearchBounds, f64)>,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;

    wtr.write_record(&[
        "Initial Time",
        "Min Departure [Days]",
        "Max Departure [Days]",
        "Min TOF [Days]",
        "Max TOF [Days]",
        "Max C3 [km^2/s^2]",
    ])
    .expect("Failed to write headers");

    for (init_time, bounds, max_c3) in data {
        wtr.write_record(&[
            init_time.as_iso8601().to_string(),
            bounds.dep_start.to_string(),
            bounds.dep_end.to_string(),
            bounds.tof_min.to_string(),
            bounds.tof_max.to_string(),
            max_c3.to_string(),
        ])
        .expect("Failed to write record")
    }
    wtr.flush()?;
    println!("Wrote metadata to CSV.");
    Ok(())
}

/* Simple Magnitude Calculation */
fn magnitude(matrix: &[f64; 3]) -> f64 {
    (matrix[0].powi(2) + matrix[1].powi(2) + matrix[2].powi(2)).sqrt()
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
