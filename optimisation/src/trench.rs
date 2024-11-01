use geo::{coord, Area, BooleanOps, Centroid, EuclideanDistance, LineString, Polygon, Point, Rotate};
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

    let mut trenches = Vec::new();
    // rotate 180 times as pattern replicates after
    for i in 0..180 {
        let rotated_trench = central_trench.rotate_around_point(i as f64, centroid);
        // cut trench to site outline
        let intersection = site_outline.boolean_op(&rotated_trench, geo::OpType::Intersection);
        let percentage_coverage = intersection.unsigned_area() / site_outline.unsigned_area() * 100.0;
        // println!("Percentage coverage: {:.2}%", percentage_coverage);
        if (1.8 < percentage_coverage) & (percentage_coverage < 2.2) {
            trenches.push(TrenchPattern::CentreLine(intersection));
        }
        // trenches.push(TrenchPattern::CentreLine(intersection));
    }
    trenches
}

fn max_distance_and_centroid(site_outline: &Polygon) -> (f64, Point) {
    let centroid = site_outline.centroid().unwrap();
    let max_distance = site_outline.exterior().points().fold(0.0, |max_distance, p| {
        let distance = centroid.euclidean_distance(&p);
        if distance > max_distance {
            distance
        } else {
            max_distance
        }
    });
    (max_distance, centroid)
}

fn create_trench_poly(
    trench_centroid: Point,
    trench_width: f64,
    trench_length: f64,
) -> Polygon<f64> {
    let trench_exterior = vec![
        coord! { x: trench_centroid.x() - trench_length / 2.0, y: trench_centroid.y() - trench_width / 2.0 },
        coord! { x: trench_centroid.x() + trench_length / 2.0, y: trench_centroid.y() - trench_width / 2.0 },
        coord! { x: trench_centroid.x() + trench_length / 2.0, y: trench_centroid.y() + trench_width / 2.0 },
        coord! { x: trench_centroid.x() - trench_length / 2.0, y: trench_centroid.y() + trench_width / 2.0 },
        coord! { x: trench_centroid.x() - trench_length / 2.0, y: trench_centroid.y() - trench_width / 2.0 },
    ];
    Polygon::new(LineString(trench_exterior), vec![])
}
