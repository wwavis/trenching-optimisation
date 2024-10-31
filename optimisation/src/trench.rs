use geo::{coord, point, BooleanOps, LineString, Polygon, Rotate};
use geojson::{Feature, Geometry, Value};
use trenching_optimisation::TrenchPattern;

pub fn new_trench_layout(trench_type: String, loe: Feature) -> Option<Vec<TrenchPattern>> {
    match loe.geometry {
        Some(ref geom) => match create_trenches(geom, &trench_type) {
            Some(trenches) => {
                return Some(trenches);
            }
            None => {
                println!("Trenches not created");
                return None;
            }
        },
        None => {
            println!("No geometry found");
            return None;
        }
    }
}

fn create_trenches(geom: &Geometry, trench_type: &String) -> Option<Vec<TrenchPattern>> {
    match geom.value {
        Value::Polygon(ref polygon) => {
            let polygon_exterior = polygon[0]
                .iter()
                .map(|c| {
                    coord! { x: c[0], y: c[1] }
                })
                .collect();
            let site_outline = Polygon::new(LineString(polygon_exterior), vec![]);
            if trench_type == "centre_line_trenching" {
                return Some(centre_line_trenching(&site_outline));
            } else {
                println!("Trench type not recognised");
                return None;
            }
        }
        _ => {
            println!("Matched some other geometry");
            return None;
        }
    }
}

fn centre_line_trenching(site_outline: &Polygon) -> Vec<TrenchPattern> {
    let (max_distance, centroid) = max_distance_and_centroid(site_outline);
    let central_trench = create_trench_poly(centroid, 2.0, max_distance * 2.0);
    let centroid = point! { x: centroid.0, y: centroid.1 };

    let mut trenches = Vec::new();
    // rotate 180 times as pattern replicates after
    for i in 0..180 {
        let rotated_trench = central_trench.rotate_around_point(i as f64, centroid);
        // cut trench to site outline
        let intersection = site_outline.boolean_op(&rotated_trench, geo::OpType::Intersection);
        // let percentage_coverage = intersection.unsigned_area() / site_outline.unsigned_area() * 100.0;
        // println!("Percentage coverage: {:.2}%", percentage_coverage);
        trenches.push(TrenchPattern::CentreLine(intersection));
    }
    trenches
}

fn max_distance_and_centroid(site_outline: &Polygon) -> (f64, (f64, f64)) {
    let (x_coords, y_coords): (Vec<f64>, Vec<f64>) = site_outline
        .exterior()
        .points()
        .map(|c| (c.x(), c.y()))
        .unzip();
    let min_x = x_coords.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = x_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_y = y_coords.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = y_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    // squareroot of sum of squares of differences
    let max_distance = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    (max_distance, ((max_x + min_x) / 2.0, (max_y + min_y) / 2.0))
}

fn create_trench_poly(
    trench_centre: (f64, f64),
    trench_width: f64,
    trench_length: f64,
) -> Polygon<f64> {
    let trench_exterior = vec![
        coord! { x: trench_centre.0 - trench_length / 2.0, y: trench_centre.1 - trench_width / 2.0 },
        coord! { x: trench_centre.0 + trench_length / 2.0, y: trench_centre.1 - trench_width / 2.0 },
        coord! { x: trench_centre.0 + trench_length / 2.0, y: trench_centre.1 + trench_width / 2.0 },
        coord! { x: trench_centre.0 - trench_length / 2.0, y: trench_centre.1 + trench_width / 2.0 },
        coord! { x: trench_centre.0 - trench_length / 2.0, y: trench_centre.1 - trench_width / 2.0 },
    ];
    Polygon::new(LineString(trench_exterior), vec![])
}
