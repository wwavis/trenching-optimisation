use geo::{
    coord, Area, BooleanOps, Centroid, EuclideanDistance, LineString, MultiPolygon, Point, Polygon,
    Rotate, Translate,
};
use geojson::{Feature, Geometry, Value};
use trenching_optimisation::{TrenchConfig, TrenchLayout};
use rayon::prelude::*;

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
            if config.layout == "centre_line" {
                return centre_line(&site_outline, config);
            } else if config.layout == "continuous" {
                return continuous(&site_outline, config);
            } else {
                panic!("Trench layout not recognised");
            }
        }
        _ => {
            panic!("LOE geometry not a polygon");
        }
    }
}

fn centre_line(site_outline: &Polygon, config: &TrenchConfig) -> Vec<TrenchLayout> {
    let (max_distance, centroid) = max_distance_and_centroid(site_outline);

    let trench_patterns = (0..180)
        .into_par_iter()
        .map(|rotation| {
            let trench = create_single_trench(centroid, config.width, max_distance * 2.0, rotation);

            // cut trench to site outline
            let intersection = site_outline.boolean_op(&trench, geo::OpType::Intersection);
            let percentage_coverage = calculate_coverage(&intersection, site_outline);

            TrenchLayout::CentreLine(intersection)
            // if (config.coverage - 0.2 <= percentage_coverage)
            //     & (percentage_coverage <= config.coverage + 0.2)
            // {
            //     return TrenchLayout::CentreLine(intersection);
            // }
        })
        .collect();
    trench_patterns
}

fn get_centroid_bounds_and_spacing(max_distance: &f64, config: &TrenchConfig) -> ((i32, i32), usize) {
    let n = (max_distance / config.spacing.unwrap()).floor() as i32;
    let spacing_i = config.spacing.unwrap() as i32;
    ((-n*spacing_i, (n+1)*spacing_i), spacing_i as usize)
}

fn continuous(site_outline: &Polygon, config: &TrenchConfig) -> Vec<TrenchLayout> {
    let (max_distance, centroid) = max_distance_and_centroid(site_outline);
    let ((start, end), spacing_i) = get_centroid_bounds_and_spacing(&max_distance, config);
    let site_outline_as_multi = MultiPolygon(vec![site_outline.clone()]);

    // rotate 180 times as layout repicates after this
    let trench_patterns = (0..180)
        .into_par_iter()
        .map(|rotation| {
            let mut trenches: Vec<Polygon> = Vec::new();

            for x_offset in (start..end).step_by(spacing_i) {
                let trench_centroid = centroid.translate(x_offset as f64, 0.0).rotate_around_point(rotation as f64, centroid);
                trenches.push(create_single_trench(trench_centroid, config.width, max_distance * 2.0, rotation));
            }
            let trench_pattern = MultiPolygon(trenches);

            // cut trench to site outline
            let intersection = site_outline_as_multi.boolean_op(&trench_pattern, geo::OpType::Intersection);
            let percentage_coverage = calculate_coverage(&intersection, site_outline);

            TrenchLayout::Continuous(intersection)
        })
        .collect();
    trench_patterns
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
