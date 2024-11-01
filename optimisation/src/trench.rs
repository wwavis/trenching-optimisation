use geo::{
    coord, Area, BooleanOps, Centroid, EuclideanDistance, LineString, MultiPolygon, Point, Polygon,
    Rotate,
};
use geojson::{Feature, Geometry, Value};
use trenching_optimisation::{TrenchConfig, TrenchLayout};

pub fn new_trench_layout(config: &TrenchConfig, loe: Feature) -> Option<Vec<TrenchLayout>> {
    match loe.geometry {
        Some(ref geom) => {
            let trenches = create_trenches(geom, config);
            Some(trenches)
        }
        None => {
            panic!("No LOE geometry found");
        }
    }
}

fn create_trenches(geom: &Geometry, config: &TrenchConfig) -> Vec<TrenchLayout> {
    match geom.value {
        Value::Polygon(ref polygon) => {
            let polygon_exterior = polygon[0]
                .iter()
                .map(|c| {
                    coord! { x: c[0], y: c[1] }
                })
                .collect();
            let site_outline = Polygon::new(LineString(polygon_exterior), vec![]);
            if config.layout == "centre_line_trenching" {
                return centre_line_trenching(&site_outline, config);
            } else {
                panic!("Trench layout not recognised");
            }
        }
        _ => {
            panic!("LOE geometry not a polygon");
        }
    }
}

fn centre_line_trenching(site_outline: &Polygon, config: &TrenchConfig) -> Vec<TrenchLayout> {
    let (max_distance, centroid) = max_distance_and_centroid(site_outline);

    let mut trenches = Vec::new();
    // rotate 180 times as layout repicates after this
    for rotation in 0..180 {
        let trench = create_single_trench(centroid, config.width, max_distance * 2.0, rotation);

        // cut trench to site outline
        let intersection = site_outline.boolean_op(&trench, geo::OpType::Intersection);
        let percentage_coverage = calculate_coverage(&intersection, site_outline);

        // trenches.push(TrenchLayout::CentreLine(intersection));
        if (config.coverage - 0.2 <= percentage_coverage)
            & (percentage_coverage <= config.coverage + 0.2)
        {
            trenches.push(TrenchLayout::CentreLine(intersection));
            continue;
        }
    }
    trenches
}

fn max_distance_and_centroid(site_outline: &Polygon) -> (f64, Point) {
    let centroid = site_outline.centroid().unwrap();
    let max_distance = site_outline
        .exterior()
        .points()
        .fold(0.0, |max_distance, p| {
            let distance = centroid.euclidean_distance(&p);
            if distance > max_distance {
                distance
            } else {
                max_distance
            }
        });
    (max_distance, centroid)
}

fn create_single_trench(centroid: Point, width: f64, length: f64, rotation: i32) -> Polygon<f64> {
    let trench_exterior = vec![
        coord! { x: centroid.x() - width / 2.0, y: centroid.y() - length / 2.0 },
        coord! { x: centroid.x() + width / 2.0, y: centroid.y() - length / 2.0 },
        coord! { x: centroid.x() + width / 2.0, y: centroid.y() + length / 2.0 },
        coord! { x: centroid.x() - width / 2.0, y: centroid.y() + length / 2.0 },
        coord! { x: centroid.x() - width / 2.0, y: centroid.y() - length / 2.0 },
    ];
    Polygon::new(LineString(trench_exterior), vec![]).rotate_around_point(rotation as f64, centroid)
}

fn calculate_coverage(trench_layout: &MultiPolygon<f64>, site_outline: &Polygon<f64>) -> f64 {
    trench_layout.unsigned_area() / site_outline.unsigned_area() * 100.0
}
