use geo::{
    coord, Area, BooleanOps, Centroid, EuclideanDistance, LineString, MultiPolygon, Point, Polygon,
    Rotate, Translate,
};
use geojson::{Geometry, Value};
use rayon::prelude::*;
use trenching_optimisation::array::{Configuration, PatternRotationAxis};
use trenching_optimisation::{
    Degree, Distribution, Rectangle, Structure, TrenchConfig, TrenchLayout,
};

pub fn create_layouts(config: &TrenchConfig, limit_of_excavation: Polygon) -> Vec<TrenchLayout> {
    // exclude holes as removed in preprocessing
    let centroid = limit_of_excavation.centroid().unwrap();
    let max_distance_from_centroid = get_max_distance_from_centroid(centroid, &limit_of_excavation);

    match config.distribution {
        Distribution::Spacing(spacing) => {
            return spacing_based_layouts(
                &limit_of_excavation,
                *config,
                max_distance_from_centroid,
                centroid,
                spacing,
            );
        }
        Distribution::Coverage(coverage) => {
            return coverage_based_layouts(
                &limit_of_excavation,
                *config,
                max_distance_from_centroid,
                centroid,
                coverage,
            );
        }
    }
}

fn get_size_of_grid(max_distance_from_centroid: &f64, spacing: &f64) -> i32 {
    (max_distance_from_centroid / spacing).floor() as i32
}

fn trench_of_array_coordinate(
    x_index: usize,
    y_index: usize,
    x_offset: i32,
    y_offset: i32,
    centroid: Point,
    spacing: f64,
    array_config: &Configuration,
    rectangle: Rectangle,
) -> Option<Polygon> {
    let trench_centroid = centroid.translate(x_offset as f64 * spacing, y_offset as f64 * spacing);
    let is_alternate_point = (x_index + y_index) % 2 == 0;
    let rotation = match array_config.pattern_rotation_axis {
        PatternRotationAxis::ByCell => {
            if is_alternate_point {
                array_config.base_angle
            } else {
                array_config.alternate_angle
            }
        }
        PatternRotationAxis::ByColumn => {
            if x_index % 2 == 0 {
                array_config.base_angle
            } else {
                array_config.alternate_angle
            }
        }
    };

    if array_config.separated & is_alternate_point {
        None
    } else {
        Some(plot_trench(
            trench_centroid,
            rectangle.width,
            rectangle.length,
            rotation,
        ))
    }
}

fn spacing_based_layouts(
    limit_of_excavation: &Polygon,
    config: TrenchConfig,
    max_distance_from_centroid: f64,
    centroid: Point,
    spacing: f64,
) -> Vec<TrenchLayout> {
    let n = get_size_of_grid(&max_distance_from_centroid, &spacing);
    let x_offsets = -n..n + 1;
    let trenches = match config.structure {
        Structure::Parallel(line) => {
            x_offsets
                .into_par_iter()
                .map(|x_offset| {
                    let trench_centroid = centroid.translate(x_offset as f64 * spacing, 0.0);
                    plot_trench(
                        trench_centroid,
                        line.width,
                        max_distance_from_centroid * 2.0,
                        Degree(0.0),
                    )
                })
                .collect()
            // TODO: test performance of this vs .push() to Vec
        }
        Structure::Array(rectangle, array_config) => {
            let y_offsets = -n..n + 1;
            x_offsets
                .into_par_iter()
                .enumerate()
                .flat_map(|(x_index, x_offset)| {
                    y_offsets
                        .clone()
                        .into_iter()
                        .enumerate()
                        .filter_map(move |(y_index, y_offset)| {
                            trench_of_array_coordinate(
                                x_index,
                                y_index,
                                x_offset,
                                y_offset,
                                centroid,
                                spacing,
                                &array_config,
                                rectangle,
                            )
                        })
                        .collect::<Vec<Polygon>>()
                })
                .collect::<Vec<Polygon>>()
        }
    };
    get_rotated_trench_patterns(
        trenches,
        config.structure.get_rotational_symmetry(),
        centroid,
        limit_of_excavation,
    )
}

fn plot_trench(
    centroid: Point,
    width: f64,
    length: f64,
    rotation: Degree,
) -> Polygon<f64> {
    let trench_exterior = vec![
        coord! { x: centroid.x() - width / 2.0, y: centroid.y() - length / 2.0 },
        coord! { x: centroid.x() + width / 2.0, y: centroid.y() - length / 2.0 },
        coord! { x: centroid.x() + width / 2.0, y: centroid.y() + length / 2.0 },
        coord! { x: centroid.x() - width / 2.0, y: centroid.y() + length / 2.0 },
        coord! { x: centroid.x() - width / 2.0, y: centroid.y() - length / 2.0 },
    ];
    Polygon::new(LineString(trench_exterior), vec![]).rotate_around_point(rotation.0, centroid)
}

fn get_max_distance_from_centroid(centroid: Point ,limit_of_excavation: &Polygon) -> f64 {
    let max_distance_from_centroid = limit_of_excavation
        .exterior()
        .points()
        .fold(0.0, |max_distance_from_centroid, p| {
            let distance = centroid.euclidean_distance(&p);
            if distance > max_distance_from_centroid {
                distance
            } else {
                max_distance_from_centroid
            }
        });
    max_distance_from_centroid
}

fn calculate_coverage(trench_layout: &MultiPolygon<f64>, limit_of_excavation: &Polygon<f64>) -> f64 {
    trench_layout.unsigned_area() / limit_of_excavation.unsigned_area() * 100.0
}

fn get_rotated_trench_patterns(
    trenches: Vec<Polygon>,
    rotations: i32,
    centroid: Point,
    limit_of_excavation: &Polygon,
) -> Vec<TrenchLayout> {
    let limit_of_excavation_as_multi = MultiPolygon(vec![limit_of_excavation.clone()]);

    let trench_patterns = (0..rotations)
        .into_par_iter()
        .map(|rotation| {
            let trench_pattern =
                MultiPolygon(trenches.clone()).rotate_around_point(rotation as f64, centroid);

            // cut trench to site outline
            let intersection =
                limit_of_excavation_as_multi.boolean_op(&trench_pattern, geo::OpType::Intersection);

            let _percentage_coverage = calculate_coverage(&intersection, limit_of_excavation);

            TrenchLayout(intersection)
        })
        .collect();
    trench_patterns
    // TODO: return average percentage coverage
}