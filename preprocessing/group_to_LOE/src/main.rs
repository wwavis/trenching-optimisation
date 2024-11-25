use anyhow::Result;
use fs_err::File;
use geo::{coord, Coord, Intersects, LineString, Polygon};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use std::io::{BufReader, BufWriter};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let loe_gj = read_raw_geojson(format!("LOE_{}", &args[1]))?;

    let mut loes: Vec<Feature> = Vec::new();
    let mut loes_polygon: Vec<Polygon> = Vec::new();

    match loe_gj {
        GeoJson::FeatureCollection(ref collection) => {
            for feature in &collection.features {
                loes.push(feature.clone());
                if let Some(ref geom) = feature.geometry {
                    match geom.value {
                        Value::Polygon(ref polygon) => {
                            let poly_exterior = polygon[0]
                                .iter()
                                .map(|c| {
                                    coord! { x: c[0], y: c[1] }
                                })
                                .collect::<Vec<Coord>>();
                            loes_polygon.push(Polygon::new(LineString(poly_exterior), vec![]));
                        }
                        _ => println!("Non Polygon GeoJSON not supported"),
                    }
                }
            }
        }
        _ => println!("Non FeatureCollection GeoJSON not supported"),
    }

    let features_gj = read_raw_geojson(format!("features_{}", &args[1]))?;
    let mut loe_feature_collections: Vec<Vec<Feature>> = vec![Vec::new(); loes.len()];

    let mut number_of_features = 0;
    if let GeoJson::FeatureCollection(ref collection) = features_gj {
        for feature in collection.features.iter() {
            if let Some(ref geom) = feature.geometry {
                for (i, loe_polygon) in loes_polygon.iter().enumerate() {
                    if compare_loe_to_feature(loe_polygon.clone(), geom) {
                        let mut feature = feature.clone();
                        // standardise the Layer names
                        if feature.contains_property("LANDSCAPE") {
                            feature.set_property("Layer", feature.property("LANDSCAPE").unwrap().clone());
                            feature.remove_property("LANDSCAPE");
                        }
                        if feature.contains_property("Phase") {
                            feature.set_property("Layer", feature.property("Phase").unwrap().clone());
                            feature.remove_property("Phase");
                        }
                        loe_feature_collections[i].push(feature);
                    }
                }
                number_of_features += 1;
            }
        }
    } else {
        println!("Non FeatureCollection GeoJSON not supported");
    }

    let number_of_features_not_in_loe = number_of_features
        - loe_feature_collections
            .iter()
            .map(|x| x.len())
            .sum::<usize>();
    println!(
        "Number of features not in LOE: {} for {}",
        number_of_features_not_in_loe, &args[1]
    );

    for (i, loe_feature_collection) in loe_feature_collections.iter().enumerate() {
        save_feature_collection_to_geojson(loe_feature_collection.clone(), &args[1], i)?;
    }

    for (i, loe) in loes.iter().enumerate() {
        save_feature_to_geojson(loe.clone(), &args[1], i)?;
    }

    Ok(())
}

fn save_feature_collection_to_geojson(
    features: Vec<Feature>,
    folder_name: &String,
    index: usize,
) -> Result<()> {
    fs_err::create_dir_all(format!(
        "../../data/grouped_by_loe/{}/{}",
        folder_name, index
    ))?;
    let file = File::create(format!(
        "../../data/grouped_by_loe/{}/{}/features.geojson",
        folder_name, index
    ))?;
    let writer = BufWriter::new(file);
    let feature_collection = FeatureCollection {
        bbox: None,
        features: features,
        foreign_members: None,
    };
    serde_json::to_writer(writer, &GeoJson::FeatureCollection(feature_collection))?;
    Ok(())
}

fn save_feature_to_geojson(feature: Feature, folder_name: &String, index: usize) -> Result<()> {
    fs_err::create_dir_all(format!(
        "../../data/grouped_by_loe/{}/{}",
        folder_name, index
    ))?;
    let file = File::create(format!(
        "../../data/grouped_by_loe/{}/{}/loe.geojson",
        folder_name, index
    ))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &GeoJson::Feature(feature))?;
    Ok(())
}

fn read_raw_geojson(file_name: String) -> Result<GeoJson> {
    let file = File::open(format!("../../data/raw_geojsons/{}.geojson", file_name))?;
    let reader = BufReader::new(file);
    let gj: GeoJson = serde_json::from_reader(reader)?;
    Ok(gj)
}

fn compare_loe_to_feature(loe_polygon: Polygon, geom: &Geometry) -> bool {
    match geom.value {
        Value::Polygon(ref polygon) => {
            let poly_exterior = polygon[0]
                .iter()
                .map(|c| {
                    coord! { x: c[0], y: c[1] }
                })
                .collect::<Vec<Coord>>();
            let poly = Polygon::new(LineString(poly_exterior), vec![]);
            if poly.intersects(&loe_polygon) {
                return true;
            } else {
                return false;
            }
        }
        _ => {
            println!("Non Polygon GeoJSON not supported");
            return false;
        }
    }
}
