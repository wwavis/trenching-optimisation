use anyhow::{anyhow, Result};
use fs_err::File;
use geo::{coord, LineString, MultiPolygon, Polygon};
use geojson::{Feature, GeoJson, Geometry, Value};
use std::io::BufReader;
use std::time::Instant;

#[derive(Debug)]
pub struct TrenchLayout(pub MultiPolygon<f64>);

#[derive(Debug)]
pub struct TestLocation {
    pub limit_of_excavation: Polygon,
    pub features: Vec<Polygon<f64>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub width: f64,
    pub length: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub width: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Degree(pub f64);

impl Degree {
    pub fn new(value: f64) -> Self {
        Degree(value)
    }
}

pub mod array {
    use crate::Degree;
    #[derive(Debug, Clone, Copy)]
    pub enum PatternRotationAxis {
        ByCell,
        ByColumn,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Configuration {
        pub base_angle: Degree,
        pub alternate_angle: Degree,
        pub pattern_rotation_axis: PatternRotationAxis,
        pub separated: bool,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Structure {
    Parallel(Line),
    Array(Rectangle, array::Configuration),
}

impl Structure {
    pub fn get_rotational_symmetry(self) -> i32 {
        match self {
            Structure::Parallel(_) => 180,
            Structure::Array(rectangle, array_configuration) => {
                if (rectangle.width == rectangle.length) & !array_configuration.separated {
                    90
                } else {
                    180
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Distribution {
    Spacing(f64),  // meters
    Coverage(f64), // percentage coverage
}

// #[derive(Debug)]
// pub enum Pattern {
//     CentreLine,
//     Continuous,
//     ParallelArray,
//     StandardGrid,
//     TestPits,
//     Herringbone,
//     // RamsgateHarbourArray,
// }

#[derive(Debug, Clone, Copy)]
pub struct TrenchConfig {
    // TODO: add shifts in x/y
    pub structure: Structure,
    pub distribution: Distribution,
}

impl TrenchConfig {
    // TODO: add centre_line
    // TODO: add validate_spacing
    // pub fn validate_spacing(width: f64, distribution: Distribution) {
    //     match distribution {
    //         Distribution::Spacing(spacing) => {
    //             assert!(width / 2.0 < spacing, "Spacing too small for width");
    //         }
    //         _ => {}
    //     }
    // }

    pub fn continuous(width: f64, distribution: Distribution) -> Self {
        match distribution {
            Distribution::Spacing(spacing) => {
                assert!(width / 2.0 < spacing, "Spacing too small for width");
            }
            _ => {}
        }
        TrenchConfig {
            structure: Structure::Parallel(Line { width }),
            distribution,
        }
    }
    pub fn parallel_array(width: f64, length: f64, distribution: Distribution) -> Self {
        match distribution {
            Distribution::Spacing(spacing) => {
                assert!(width / 2.0 < spacing, "Spacing too small for width");
                assert!(length / 2.0 < spacing, "Spacing too small for length");
            }
            _ => {}
        }
        TrenchConfig {
            structure: Structure::Array(
                Rectangle { width, length },
                array::Configuration {
                    base_angle: Degree::new(0.0),
                    alternate_angle: Degree::new(0.0),
                    pattern_rotation_axis: array::PatternRotationAxis::ByCell,
                    separated: true,
                },
            ),
            distribution,
        }
    }
    pub fn standard_grid(width: f64, length: f64, distribution: Distribution) -> Self {
        match distribution {
            Distribution::Spacing(spacing) => {
                assert!(
                    width / 2.0 + length / 2.0 < spacing,
                    "Spacing too small for width and legnth"
                );
            }
            _ => {}
        }
        TrenchConfig {
            structure: Structure::Array(
                Rectangle { width, length },
                array::Configuration {
                    base_angle: Degree::new(0.0),
                    alternate_angle: Degree::new(90.0),
                    pattern_rotation_axis: array::PatternRotationAxis::ByCell,
                    separated: false,
                },
            ),
            distribution,
        }
    }
    pub fn test_pits(width: f64, distribution: Distribution) -> Self {
        match distribution {
            Distribution::Spacing(spacing) => {
                assert!(width / 2.0 < spacing, "Spacing too small for width");
            }
            _ => {}
        }
        TrenchConfig {
            structure: Structure::Array(
                Rectangle {
                    width,
                    length: width,
                },
                array::Configuration {
                    base_angle: Degree::new(0.0),
                    alternate_angle: Degree::new(0.0),
                    pattern_rotation_axis: array::PatternRotationAxis::ByCell,
                    separated: false,
                },
            ),
            distribution,
        }
    }
    pub fn herringbone(width: f64, length: f64, distribution: Distribution) -> Self {
        // TODO: add validation for herringbone
        // match distribution {
        //     Distribution::Spacing(spacing) => {
        //         assert!(
        //             width / 2.0 + length / 2.0 < spacing,
        //             "Spacing too small for width and legnth"
        //         );
        //     }
        //     _ => {}
        // }
        TrenchConfig {
            structure: Structure::Array(
                Rectangle { width, length },
                array::Configuration {
                    base_angle: Degree::new(45.0),
                    alternate_angle: Degree::new(315.0),
                    pattern_rotation_axis: array::PatternRotationAxis::ByColumn,
                    separated: false,
                },
            ),
            distribution,
        }
    }
    // pub fn centre_line_of_width(width: f64) -> Self {
    //     TrenchConfig {
    //         structure: Structure::Parallel(Line { width }),
    //         distribution: Distribution::Spacing(0.0),
    //     }
    // }

    // pub fn centre_line_coverage(coverage: f64) -> Self {
    //     TrenchConfig {
    //         structure: Structure::Parallel(Line { width: 0.0 }),
    //         distribution: Distribution::Coverage(coverage),
    //     }
    // }
}

pub fn read_single_test_location_data(
    site_name: String,
    loe_i: String,
    selected_layer: Option<&str>,
) -> Result<TestLocation> {
    let now = Instant::now();
    let limit_of_excavation = read_single_loe_feature(site_name.clone(), loe_i.clone())?;
    let gj = read_single_features_geojson(site_name.clone(), loe_i.clone())?;
    match process_geojson(&gj, selected_layer) {
        Some(features) => {
            println!("Reading files took: {:?}", now.elapsed());
            Ok(TestLocation { limit_of_excavation, features })
        }
        None => Err(anyhow!(
            "No {:?} at site: {} location: {}",
            selected_layer,
            site_name,
            loe_i
        )),
    }
}

fn read_single_features_geojson(site_name: String, loe_i: String) -> Result<GeoJson> {
    let file = File::open(format!(
        "../data/grouped_by_loe/{}/{}/features.geojson",
        site_name, loe_i
    ))?;
    let reader = BufReader::new(file);
    let gj: GeoJson = serde_json::from_reader(reader)?;
    Ok(gj)
}

fn get_site_outline_of_loe(polygon: Vec<Vec<Vec<f64>>>) -> Polygon {
    if polygon.len() > 1 {
        println!("Warning: more than one polygon found for LOE");
    }
    let poly_exterior = polygon[0]
        .iter()
        .map(|c| {
            coord! { x: c[0], y: c[1] }
        })
        .collect();
    Polygon::new(LineString(poly_exterior), vec![])
}

fn read_single_loe_feature(site_name: String, loe_i: String) -> Result<Polygon> {
    let file = File::open(format!(
        "../data/grouped_by_loe/{}/{}/loe.geojson",
        site_name, loe_i
    ))?;
    let reader = BufReader::new(file);
    let feature: Feature = serde_json::from_reader(reader)?;
    match feature.geometry {
        Some(geometry) => {
            match geometry.value {
                Value::Polygon(polygon) => {
                    Ok(get_site_outline_of_loe(polygon))
                }
                _ => {
                    return Err(anyhow!("Geometry is not a polygon"));
                }
            }
        }
        // Ok(geometry),
        None => Err(anyhow!("No geometry found in LOE file")),
    }
}

pub fn read_all_test_location_data(selected_layer: Option<&str>) -> Result<Vec<TestLocation>> {
    let now = Instant::now();
    let mut test_locations = Vec::new();

    let sites_location_counts = [
        ("Stansted", 17),
        ("Heathrow", 5),
        ("A355_BeaconsfieldEasternReliefRoad", 3),
        ("_NDR__", 22),
        ("wingerworth", 2),
    ];
    for (site, location_count) in sites_location_counts.iter() {
        for i in 0..*location_count {
            let limit_of_excavation = read_single_loe_feature(site.to_string(), i.to_string())?;
            let features = read_single_features_geojson(site.to_string(), i.to_string())?;
            match process_geojson(&features, selected_layer) {
                Some(polygons) => {
                    test_locations.push(TestLocation {
                        limit_of_excavation: limit_of_excavation,
                        features: polygons,
                    });
                }
                None => {
                    // println!("Unable to make polygons for site: {} location: {}", site, i);
                }
            }
        }
    }
    println!("Reading files took: {:?}", now.elapsed());
    Ok(test_locations)
}

fn process_geojson(gj: &GeoJson, selected_layer: Option<&str>) -> Option<Vec<Polygon<f64>>> {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => {
            let mut polygons = Vec::new();
            for feature in &collection.features {
                match selected_layer {
                    // Skip features that don't match the selected layer
                    Some(layer) => {
                        if feature.property("Layer").unwrap() != layer {
                            continue;
                        }
                    }
                    None => {}
                }
                if let Some(ref geom) = feature.geometry {
                    if let Some(poly) = geometry_to_polygon(geom) {
                        polygons.push(poly);
                    } else {
                        println!("No polygon found");
                    }
                }
            }
            if polygons.is_empty() {
                None
            } else {
                Some(polygons)
            }
        }
        _ => {
            println!("Non FeatureCollection GeoJSON not supported");
            None
        }
    }
}

// Process GeoJSON geometries
fn geometry_to_polygon(geom: &Geometry) -> Option<Polygon<f64>> {
    match geom.value {
        Value::Polygon(ref polygon) => {
            let poly_exterior = polygon[0]
                .iter()
                .map(|c| {
                    coord! { x: c[0], y: c[1] }
                })
                .collect();
            Some(Polygon::new(LineString(poly_exterior), vec![]))
        }
        _ => {
            // TODO: update this placeholder for other geometry types
            println!("Matched some other geometry");
            None
        }
    }
}
