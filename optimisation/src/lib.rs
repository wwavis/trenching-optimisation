use anyhow::Result;
use fs_err::File;
use geo::MultiPolygon;
use geojson::{Feature, GeoJson};
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
    pub features: GeoJson,
}

pub fn read_single_features_geojson(site_name: String, loe_i: String) -> Result<GeoJson> {
    let file = File::open(format!(
        "../data/grouped_by_loe/{}/{}/features.geojson",
        site_name, loe_i
    ))?;
    let reader = BufReader::new(file);
    let gj: GeoJson = serde_json::from_reader(reader)?;
    Ok(gj)
}

pub fn read_single_loe_feature(site_name: String, loe_i: String) -> Result<Feature> {
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
            test_locations.push(TestLocation { loe, features });
        }
    }
    println!("Reading files took: {:?}", now.elapsed());
    Ok(test_locations)
}
