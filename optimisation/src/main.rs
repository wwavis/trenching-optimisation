mod intersects;
use trenching_optimisation::read_geojson;
use geojson::{GeoJson, Geometry, Value};
use geo::{Polygon, LineString, Coord, coord};

fn main() {
    println!("Solve the trenching optimisation problem");
    // intersects::test();
    let gj = read_geojson().unwrap();
    process_geojson(&gj);
}

fn process_geojson(gj: &GeoJson) {
    match *gj {
        GeoJson::FeatureCollection(ref ctn) => {
            let mut matched = 0;
            let mut unmatched = 0;
            for feature in &ctn.features {
                if let Some(ref geom) = feature.geometry {
                    if match_geometry(geom) {
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

/// Process GeoJSON geometries
fn match_geometry(geom: &Geometry) -> bool {
    match geom.value {
        Value::Polygon(ref polygon) => {
            let poly1 = polygon[0].iter().map(|c| {
                coord! { x: c[0], y: c[1] }
            }).collect::<Vec<Coord>>();
            let poly = Polygon::new(LineString(poly1), vec![]);
            if intersects::test(poly) {
                true
            } else {
                false
            }

        },
        _ => {
            // TODO: update this placeholder for other geometry types
            println!("Matched some other geometry");
            false 
        },
    }
}