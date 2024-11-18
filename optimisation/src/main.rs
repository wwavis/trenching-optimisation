mod intersects;
mod trench;

use geo::Polygon;
use rayon::prelude::*;
use std::time::Instant;

use trenching_optimisation::{
    read_all_test_location_data, read_single_test_location_data, TrenchConfig, TrenchLayout,
};

fn main() {
    // TODO: get better name than selected_config doesn't sit right
    let centre_line = TrenchConfig {
        layout: "centre_line".to_string(),
        width: 2.0,
        length: None,
        spacing: None,
        coverage: 2.0,
    };
    let continuous = TrenchConfig {
        layout: "continuous".to_string(),
        width: 2.0,
        length: None,
        spacing: Some(20.0),
        coverage: 2.0,
    };
    run_on_single_loe(&continuous, "Heathrow".to_string(), "4".to_string());
    run_on_all_loes(&continuous);
}

fn count_features_hit_or_missed(
    features: &Vec<Polygon<f64>>,
    trenches: &TrenchLayout,
) -> (i32, i32) {
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

fn run_on_single_loe(selected_config: &TrenchConfig, site_name: String, loe_i: String) {
    println!("Running on single LOE");
    let test_location = read_single_test_location_data(site_name, loe_i).unwrap();

    let now = Instant::now();
    let trenches = trench::new_trench_layout(selected_config, test_location.loe);
    println!("Creating trenches took: {:?}", now.elapsed());

    let now = Instant::now();
    let _: Vec<(i32, i32)> = trenches
        .unwrap()
        .into_par_iter()
        .map(|trench| {
            let (features_found, features_missed) =
                count_features_hit_or_missed(&test_location.features, &trench);
            (features_found, features_missed)
        })
        .collect();
    println!("Calculating features hit took: {:?}", now.elapsed());
}

fn run_on_all_loes(selected_config: &TrenchConfig) {
    println!("\nRunning on all LOEs");
    let test_locations = read_all_test_location_data().unwrap();

    let now = Instant::now();

    let mut total_found = 0;
    let mut total_missed = 0;
    let mut total_trenches = 0;

    for test_location in test_locations {
        let trenches = trench::new_trench_layout(selected_config, test_location.loe);
        let found_or_missed: Vec<(i32, i32)> = trenches
            .unwrap()
            .into_par_iter()
            .map(|trench| {
                let (features_found, features_missed) =
                    count_features_hit_or_missed(&test_location.features, &trench);
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
        "Testing {}m wide {}",
        selected_config.width, selected_config.layout
    );
    println!(
        "Total features found: {}, total features missed: {}, percentage found: {:.2}%",
        total_found, total_missed, percentage_found
    );
    println!("Total trenches tested: {}", total_trenches);
    println!("Testing took: {:?}", now.elapsed());
}
