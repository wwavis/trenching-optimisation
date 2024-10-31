mod intersects;
mod trench;

use geo::{coord, LineString, Polygon};
use geojson::{GeoJson, Geometry, Value};
use rayon::prelude::*;
use std::time::Instant;

use trenching_optimisation::{
    read_all_test_location_data, read_single_features_geojson, read_single_loe_feature,
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
        let features = test_location.features;
        let found_or_missed: Vec<(i32, i32)> = trenches
            .unwrap()
            .into_par_iter()
            .map(|trench| {
                let (features_found, features_missed) = process_geojson(&features, &trench);
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
    // let site_name = format!("Stansted");
    // let loe_i = format!("{}", 0);

    // let loe = read_single_loe_feature(site_name.clone(), loe_i.clone()).unwrap();

    // let trenches = trench::new_trench_layout("centre_line_trenching".to_string(), loe);
    // let features = read_single_features_geojson(site_name, loe_i).unwrap();
    // // process_geojson(&features, &trenches.unwrap());
    // let results: Vec<(i32, i32)> = trenches
    //     .unwrap()
    //     .into_iter()
    //     .map(|trench| {
    //         let (features_found, features_missed) = process_geojson(&features, &trench);
    //         (features_found, features_missed)
    //     })
    //     .collect();

    // let elapsed_time = now.elapsed();
    // println!("Elapsed time: {:?}", elapsed_time);
}

fn process_geojson(gj: &GeoJson, trenches: &TrenchPattern) -> (i32, i32) {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => {
            let mut features_found = 0;
            let mut features_missed = 0;
            for feature in &collection.features {
                if let Some(ref geom) = feature.geometry {
                    if match_geometry(geom, &trenches) {
                        features_found += 1;
                    } else {
                        features_missed += 1;
                    }
                }
            }
            // println!(
            //     "Features found: {}, features missed: {}",
            //     features_found, features_missed
            // );
            (features_found, features_missed)
        }
        _ => {
            println!("Non FeatureCollection GeoJSON not supported");
            (0, 0)
        }
    }
}

// Process GeoJSON geometries
fn match_geometry(geom: &Geometry, trenches: &TrenchPattern) -> bool {
    match geom.value {
        Value::Polygon(ref polygon) => {
            let poly1 = polygon[0]
                .iter()
                .map(|c| {
                    coord! { x: c[0], y: c[1] }
                })
                .collect();
            let poly = Polygon::new(LineString(poly1), vec![]);
            if intersects::test(poly, trenches) {
                true
            } else {
                false
            }
        }
        _ => {
            // TODO: update this placeholder for other geometry types
            println!("Matched some other geometry");
            false
        }
    }
}
