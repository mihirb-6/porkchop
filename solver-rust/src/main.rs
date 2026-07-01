use csv::Writer;
use satkit::Instant;
use satkit::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

mod elements;
mod trajectory;

use crate::elements::{find_periods, hohmann_transfer_time};
use crate::trajectory::{SearchBounds, find_trajectories};

// Part of the config.toml parse
#[derive(Deserialize, Debug)]
struct Config {
    rust_config: RustConfig,
}

// Another part of config.toml, this struct defines what parameters can change in the toml
#[derive(Deserialize, Debug)]
struct RustConfig {
    initial_time: (i32, i32, i32),
    departure_object: String,
    arrival_object: String,
    search_limits: (f64, f64, f64, f64, f64),
    max_c3: f64,
    dla_limits: (f64, f64),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_dir = env::current_dir()?;
    config_dir.pop();

    config_dir = config_dir.join("config.toml");
    // Read config.toml file
    let file_content = fs::read_to_string(config_dir).expect("Failed to read config file");
    let config: Config = toml::from_str(&file_content).expect("Failed to parse TOML");

    // Define year, month, day from config
    let (yyyy, mm, dd) = config.rust_config.initial_time;

    // Define launch and arrival planet from config
    let dep_obj = config.rust_config.departure_object;
    let arr_obj = config.rust_config.arrival_object;

    // Define initial time
    let initial_time = Instant::from_date(yyyy, mm, dd).unwrap();

    // Dictionary to store key-value pair of planet and satkit's solar system object
    let pl_hashmap: HashMap<String, SolarSystem> = HashMap::from([
        (String::from("mercury"), SolarSystem::Mercury),
        (String::from("venus"), SolarSystem::Venus),
        (String::from("earth"), SolarSystem::EMB),
        (String::from("mars"), SolarSystem::Mars),
        (String::from("jupiter"), SolarSystem::Jupiter),
        (String::from("saturn"), SolarSystem::Saturn),
        (String::from("uranus"), SolarSystem::Uranus),
        (String::from("neptune"), SolarSystem::Neptune),
    ]);

    // Define Departure and Arrival Locations
    let departure_object = pl_hashmap[&dep_obj];
    let arrival_object = pl_hashmap[&arr_obj];

    // Limit C3 values displayed on the plot
    let max_c3 = config.rust_config.max_c3;

    // Limit dla
    let (min_dla, max_dla) = config.rust_config.dla_limits;

    // Limit rla
    //let min_rla = -360_f64;
    //let max_rla = 360_f64;

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

    // Hohmann Transfer Time
    let hohmann_t = hohmann_transfer_time(departure_object, arrival_object, initial_time);
    println!("Hohmann Transfer Time: {:.2} days...", hohmann_t);

    println!("===========================================================");

    // Define search limits
    let (start, end, min, max, step) = config.rust_config.search_limits;

    // Construct search
    let search = SearchBounds {
        dep_start: (start * syn_p) as i32,
        dep_end: (end * syn_p) as i32,
        tof_min: (min * hohmann_t) as i32,
        tof_max: (max * hohmann_t) as i32,
        step_size: step,
    };

    // For the print statement below
    let d_init = initial_time + satkit::Duration::from_days(search.dep_start as f64);
    let d_end = initial_time + satkit::Duration::from_days(search.dep_end as f64);

    // Prints departure dates
    println!(
        "Departure range {} - {}",
        d_init.strftime("%B-%d-%Y %H:%M:%S").unwrap(),
        d_end.strftime("%B-%d-%Y %H:%M:%S").unwrap(),
    );
    //Prints range of TOF
    println!(
        "TOF range [{:.2} - {:.2}] days",
        search.tof_min, search.tof_max
    );
    println!("===========================================================");

    // Calculate All Trajectories and Compute Delta-V
    // dep_obj, arr_obj, min_dep, max_dep, min_tof, max_tof, dep_step_size
    let (mut type1_data, mut type2_data) =
        find_trajectories(initial_time, departure_object, arrival_object, search);

    // Clip/clamp values to make contours easier to plot in python
    type1_data = type1_data
        .iter()
        .map(|(dep_date, arr_date, dep_c3, arr_c3, dla, rla)| {
            (
                *dep_date,
                *arr_date,
                (*dep_c3).clamp(0.0, max_c3),
                (*arr_c3).clamp(0.0, max_c3),
                (*dla).clamp(min_dla, max_dla),
                *rla,
            )
        })
        .collect();
    // Do same for type II data
    type2_data = type2_data
        .iter()
        .map(|(dep_date, arr_date, dep_c3, arr_c3, dla, rla)| {
            (
                *dep_date,
                *arr_date,
                (*dep_c3).clamp(0.0, max_c3),
                (*arr_c3).clamp(0.0, max_c3),
                (*dla).clamp(min_dla, max_dla),
                *rla,
            )
        })
        .collect();

    let mut plot_dir = env::current_dir()?;
    plot_dir.pop();
    plot_dir = plot_dir.join("plotter-python");

    // Write data to separate csv's
    let type1_path = plot_dir.join("TYPEI_DATA.csv");
    let type2_path = plot_dir.join("TYPEII_DATA.csv");
    let meta_path = plot_dir.join("METADATA.csv");
    //let type1_path = "/Users/mihir/projects/porkchop/plotter-python/TYPEI_DATA.csv";
    //let type2_path = "/Users/mihir/projects/porkchop/plotter-python/TYPEII_DATA.csv";
    // let meta_path = "/Users/mihir/projects/porkchop/plotter-python/METADATA.csv";

    write_to_csv(type1_path, &type1_data).unwrap();
    write_to_csv(type2_path, &type2_data).unwrap();

    // Write "metadata"/misc values that help with plotting in Python
    let mut metadata_vector: Vec<(Instant, SearchBounds, f64)> = Vec::new();
    metadata_vector.push((initial_time, search.clone(), max_c3));
    write_metadata_to_csv(meta_path, &metadata_vector).unwrap();

    Ok(())
}

/* Helper Funcion to write trajectory data to a CSV file */
fn write_to_csv(
    path: PathBuf, //&'static str,
    data: &Vec<(Instant, Instant, f64, f64, f64, f64)>,
) -> Result<(), Box<dyn Error>> {
    println!("Writing {}...", path.display());

    let mut wtr = Writer::from_path(path)?;

    wtr.write_record(&[
        "Departure Date [JD]",
        "Arrival Date [JD]",
        "Departure C3 [km^2/s^2]",
        "Arrival C3 [km^2/s^2]",
        "Departure Launch Asymptote [deg]",
        "Arrival Launch Asymptote [deg]",
    ])
    .expect("Failed to write headers");

    for (dep_date, arr_date, dep_c3, arr_c3, dla, rla) in data {
        wtr.write_record(&[
            dep_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            arr_date.as_jd_with_scale(TimeScale::UTC).to_string(),
            dep_c3.to_string(),
            arr_c3.to_string(),
            dla.to_string(),
            rla.to_string(),
        ])
        .expect("Failed to write record")
    }

    wtr.flush()?;
    println!("Wrote data to CSV.");
    Ok(())
}

/* Similar helper function to write misc values to a metadata CSV file */
fn write_metadata_to_csv(
    path: PathBuf, //&'static str,
    data: &Vec<(Instant, SearchBounds, f64)>,
) -> Result<(), Box<dyn Error>> {
    println!("Writing {}...", path.display());

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
