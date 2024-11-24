use geo::{
    coord, Area, BooleanOps, Centroid, EuclideanDistance, LineString, MultiPolygon, Point, Polygon,
    Rotate, Translate,
};
use geojson::{Feature, Geometry, Value};
use rayon::prelude::*;
use trenching_optimisation::{Layout, TrenchConfig, TrenchLayout};

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
            match config.layout {
                Layout::CentreLine => {
                    return centre_line(&site_outline, config);
                }
                Layout::Continuous => {
                    return continuous(&site_outline, config);
                }
                Layout::ParallelArray => {
                    return parallel_array(&site_outline, config);
                }
                _ => {
                    panic!("Trench layout: {:?} not recognised", config.layout);
                }
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
            let _percentage_coverage = calculate_coverage(&intersection, site_outline);

            TrenchLayout(intersection)
            // if (config.coverage - 0.2 <= percentage_coverage)
            //     & (percentage_coverage <= config.coverage + 0.2)
            // {
            //     return TrenchLayout::CentreLine(intersection);
            // }
        })
        .collect();
    trench_patterns
}

fn continuous(site_outline: &Polygon, config: &TrenchConfig) -> Vec<TrenchLayout> {
    let (max_distance, centroid) = max_distance_and_centroid(site_outline);
    let ((start, end), spacing) =
        get_centroid_bounds_and_spacing(&max_distance, config.spacing.unwrap());

    let mut trenches: Vec<Polygon> = Vec::new();

    for x_offset in (start..end).step_by(spacing) {
        let trench_centroid = centroid.translate(x_offset as f64, 0.0);
        trenches.push(create_single_trench(
            trench_centroid,
            config.width,
            max_distance * 2.0,
            0,
        ));
    }
    get_rotated_trench_patterns(trenches, 180, centroid, site_outline)
}

fn parallel_array(site_outline: &Polygon, config: &TrenchConfig) -> Vec<TrenchLayout> {
    let (max_distance, centroid) = max_distance_and_centroid(site_outline);
    let ((x_start, x_end), x_spacing) =
        get_centroid_bounds_and_spacing(&max_distance, config.spacing.unwrap());
    let ((y_start, y_end), y_spacing) =
        get_centroid_bounds_and_spacing(&max_distance, config.length.unwrap());

    let mut trenches: Vec<Polygon> = Vec::new();
    let mut skip_first_trench_in_y = true;
    for x_offset in (x_start..x_end).step_by(x_spacing) {
        let mut trench_here = true;
        let y_start_alternate = if skip_first_trench_in_y {
            y_start + y_spacing as i32
        } else {
            y_start
        };
        for y_offset in (y_start_alternate..y_end).step_by(y_spacing) {
            if trench_here {
                let trench_centroid = centroid.translate(x_offset as f64, y_offset as f64);
                trenches.push(create_single_trench(
                    trench_centroid,
                    config.width,
                    config.length.unwrap(),
                    0,
                ));
            }
            // alternate between trench and no trench
            trench_here = !trench_here;
        }
        skip_first_trench_in_y = !skip_first_trench_in_y;
    }
    get_rotated_trench_patterns(trenches, 180, centroid, site_outline)
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

fn get_centroid_bounds_and_spacing(max_distance: &f64, spacing: f64) -> ((i32, i32), usize) {
    let n = (max_distance / spacing).floor() as i32;
    let spacing = spacing as i32;
    ((-n * spacing, (n + 1) * spacing), spacing as usize)
}

fn calculate_coverage(trench_layout: &MultiPolygon<f64>, site_outline: &Polygon<f64>) -> f64 {
    trench_layout.unsigned_area() / site_outline.unsigned_area() * 100.0
}

fn get_rotated_trench_patterns(
    trenches: Vec<Polygon>,
    rotations: i32,
    centroid: Point,
    site_outline: &Polygon,
) -> Vec<TrenchLayout> {
    let site_outline_as_multi = MultiPolygon(vec![site_outline.clone()]);

    let trench_patterns = (0..rotations)
        .into_par_iter()
        .map(|rotation| {
            let trench_pattern =
                MultiPolygon(trenches.clone()).rotate_around_point(rotation as f64, centroid);

            // cut trench to site outline
            let intersection =
                site_outline_as_multi.boolean_op(&trench_pattern, geo::OpType::Intersection);

            let _percentage_coverage = calculate_coverage(&intersection, site_outline);

            TrenchLayout(intersection)
        })
        .collect();
    trench_patterns
}
