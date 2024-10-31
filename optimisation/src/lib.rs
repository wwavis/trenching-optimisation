use anyhow::Result;
use fs_err::File;
use geo::{coord, LineString, MultiPolygon, Polygon};
use geojson::{Feature, GeoJson, Geometry, Value};
use std::io::BufReader;
use std::time::Instant;

#[derive(Debug)]
pub enum TrenchPattern {
    CentreLine(MultiPolygon<f64>),
    Continuous(MultiPolygon<f64>),
}

#[derive(Debug)]
pub struct TestLocation {
    pub loe: Feature,
    pub features: Vec<Polygon<f64>>,
}

pub fn read_single_test_location_data(site_name: String, loe_i: String) -> Result<TestLocation> {
    let loe = read_single_loe_feature(site_name.clone(), loe_i.clone())?;
    let gj = read_single_features_geojson(site_name, loe_i)?;
    let features = process_geojson(&gj).unwrap();
    Ok(TestLocation {
        loe: loe,
        features: features,
    })
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

fn read_single_loe_feature(site_name: String, loe_i: String) -> Result<Feature> {
    let file = File::open(format!(
        "../data/grouped_by_loe/{}/{}/loe.geojson",
        site_name, loe_i
    ))?;
    let reader = BufReader::new(file);
    let feature: Feature = serde_json::from_reader(reader)?;
    Ok(feature)
}

pub fn read_all_test_location_data() -> Result<Vec<TestLocation>> {
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
            let loe = read_single_loe_feature(site.to_string(), i.to_string())?;
            let features = read_single_features_geojson(site.to_string(), i.to_string())?;
            match process_geojson(&features) {
                Some(polygons) => {
                    test_locations.push(TestLocation {
                        loe: loe,
                        features: polygons,
                    });
                }
                None => {
                    println!("Unable to make polygons for site: {} location: {}", site, i);
                }
            }

        }
    }
    println!("Reading files took: {:?}", now.elapsed());
    Ok(test_locations)
}

fn process_geojson(gj: &GeoJson) -> Option<Vec<Polygon<f64>>> {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => {
            let mut polygons = Vec::new();
            for feature in &collection.features {
                if let Some(ref geom) = feature.geometry {
                    if let Some(poly) = geometry_to_polygon(geom) {
                        polygons.push(poly);
                    } else {
                        println!("No polygon found");
                    }
                }
            }
            Some(polygons)
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

