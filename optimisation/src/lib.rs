use anyhow::Result;
use fs_err::File;
use geo::MultiPolygon;
use geojson::{Feature, GeoJson};
use std::io::BufReader;

#[derive(Debug)]
pub enum TrenchPattern {
    CentreLine(MultiPolygon<f64>),
    Continuous(MultiPolygon<f64>),
}

pub fn read_features_geojson(site_name: String, loe_i: String) -> Result<GeoJson> {
    let file = File::open(format!(
        "../data/grouped_by_loe/{}/{}/features.geojson",
        site_name, loe_i
    ))?;    let reader = BufReader::new(file);
    let gj: GeoJson = serde_json::from_reader(reader)?;
    Ok(gj)
}

pub fn read_loe_feature(site_name: String, loe_i: String) -> Result<Feature> {
    let file = File::open(format!(
        "../data/grouped_by_loe/{}/{}/loe.geojson",
        site_name, loe_i
    ))?;
    let reader = BufReader::new(file);
    let feature: Feature = serde_json::from_reader(reader)?;
    Ok(feature)
}
