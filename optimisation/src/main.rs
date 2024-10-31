mod intersects;
mod trench;

use geo::Polygon;
use rayon::prelude::*;
use std::time::Instant;

use trenching_optimisation::{
    read_all_test_location_data, read_single_test_location_data,
    TrenchPattern,
};

fn main() {
    let trench_type = "centre_line_trenching";
    let test_locations = read_all_test_location_data().unwrap();

    let now = Instant::now();

    let mut total_found = 0;
    let mut total_missed = 0;
    let mut total_trenches = 0;

    for test_location in test_locations {
        let trenches = trench::new_trench_layout(trench_type.to_string(), test_location.loe);
        let found_or_missed: Vec<(i32, i32)> = trenches
            .unwrap()
            .into_par_iter()
            .map(|trench| {
                let (features_found, features_missed) = count_features_hit_or_missed(&test_location.features, &trench);
                (features_found, features_missed)
            })
            .collect();
        for (found, missed) in found_or_missed {
            total_found += found;
            total_missed += missed;
            total_trenches += 1;
        }
    }

    let percentage_found = total_found as f64 / (total_found + total_missed) as f64 * 100.0;

    println!(
        "Total features found: {}, total features missed: {}, percentage found: {:.2}%",
        total_found, total_missed, percentage_found
    );

    println!("Total trenches tested: {}", total_trenches);

    let elapsed_time = now.elapsed();
    println!("Testing {} took: {:?}", trench_type, elapsed_time);

    // // Below is for a single LOE
    // let now = Instant::now();
    // let site_name = format!("Stansted");
    // let loe_i = format!("{}", 0);

    // let test_location = read_single_test_location_data(site_name, loe_i).unwrap();

    // println!("Elapsed time: {:?}", now.elapsed());
    // let trenches = trench::new_trench_layout(trench_type.to_string(), test_location.loe);
    // // process_geojson(&features, &trenches.unwrap());
    // println!("Elapsed time: {:?}", now.elapsed());
    // let found_or_missed: Vec<(i32, i32)> = trenches
    //         .unwrap()
    //         .into_par_iter()
    //         .map(|trench| {
    //             let (features_found, features_missed) = count_features_hit_or_missed(&test_location.features, &trench);
    //             (features_found, features_missed)
    //         })
    //         .collect();

    // println!("Elapsed time: {:?}", now.elapsed());
}

fn count_features_hit_or_missed(features: &Vec<Polygon<f64>>, trenches: &TrenchPattern) -> (i32, i32) {
    let mut features_found = 0;
    let mut features_missed = 0;
    for feature in features {
        if intersects::test(feature, trenches) {
            features_found += 1;
        } else {
            features_missed += 1;
        }
    }
    (features_found, features_missed)
}