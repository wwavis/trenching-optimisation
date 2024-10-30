mod intersects;
mod trench;

use geo::{coord, Coord, LineString, Polygon};
use geojson::{GeoJson, Geometry, Value};
use std::time::Instant;
use trenching_optimisation::{read_features_geojson, TrenchPattern};

fn main() {
    let now = Instant::now();

    let site_name = format!("wingerworth");
    let loe_i = format!("{}", 0);

    let trenches = trench::new_trench_layout("centre_line_trenching".to_string(), &site_name, &loe_i);
    let features = read_features_geojson(site_name, loe_i).unwrap();
    process_geojson(&features, &trenches.unwrap());

    let elapsed_time = now.elapsed();
    println!("Elapsed time: {:?}", elapsed_time);
}

fn process_geojson(gj: &GeoJson, trenches: &TrenchPattern) {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => {
            let mut matched = 0;
            let mut unmatched = 0;
            for feature in &collection.features {
                if let Some(ref geom) = feature.geometry {
                    if match_geometry(geom, &trenches) {
                        matched += 1;
                    } else {
                        unmatched += 1;
                    }
                }
            }
            println!("Matched: {}, Unmatched: {}", matched, unmatched);
        }
        _ => println!("Non FeatureCollection GeoJSON not supported"),
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
                .collect::<Vec<Coord>>();
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
