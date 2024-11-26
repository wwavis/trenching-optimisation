mod intersects;
mod trench;

use geo::Polygon;
use rayon::prelude::*;
use std::time::Instant;

use trenching_optimisation::{
    read_all_test_location_data, read_single_test_location_data, TrenchConfig, TrenchLayout,
};

fn main() {
    let centre_line = TrenchConfig::centre_line(2.0, 2.0);
    let continuous = TrenchConfig::continuous(2.0, 20.0, 2.0);
    let parallel_array = TrenchConfig::parallel_array(2.0, 30.0, 20.0, 2.0);
    let standard_grid = TrenchConfig::standard_grid(2.0, 30.0, 20.0, 2.0);
    let grid_with_wide_trenches = TrenchConfig::standard_grid(4.0, 30.0, 40.0, 2.0);
    let grid_wtth_short_trenches = TrenchConfig::standard_grid(2.0, 20.0, 30.0, 2.0);
    let test_pits = TrenchConfig::test_pits(1.0, 20.0, 2.0);

    let selected_layer = Some("Middle Bronze Age");

    for config in [
        centre_line,
        continuous,
        parallel_array,
        standard_grid,
        grid_with_wide_trenches,
        grid_wtth_short_trenches,
        test_pits,
    ]
    .iter()
    {
        run_on_single_loe(
            &config,
            "Stansted".to_string(),
            "0".to_string(),
            selected_layer,
        );
        run_on_all_loes(&config, selected_layer);
    }
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

fn run_on_single_loe(
    config: &TrenchConfig,
    site_name: String,
    loe_i: String,
    selected_layer: Option<&str>,
) {
    println!("\nRunning {:?} on single LOE", config.layout);
    let test_location = read_single_test_location_data(site_name, loe_i, selected_layer);
    match test_location {
        Ok(test_location) => {
            let now = Instant::now();
            let trenches = trench::new_trench_layout(config, test_location.loe);
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
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn run_on_all_loes(config: &TrenchConfig, selected_layer: Option<&str>) {
    println!("\nRunning {:?} on all LOEs", config.layout);
    let test_locations = read_all_test_location_data(selected_layer).unwrap();

    let now = Instant::now();

    let mut total_found = 0;
    let mut total_missed = 0;
    let mut total_trenches = 0;

    for test_location in test_locations {
        let trenches = trench::new_trench_layout(config, test_location.loe);
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
        "Testing {}m wide {:?} trenches",
        config.width, config.layout
    );
    println!(
        "Total features found: {}, total features missed: {}, percentage found: {:.2}%",
        total_found, total_missed, percentage_found
    );
    println!("Total trenches tested: {}", total_trenches);
    println!("Testing took: {:?}", now.elapsed());
}
