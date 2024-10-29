use geojson::GeoJson;
use anyhow::Result;
use fs_err::File;
use std::io::BufReader;

pub fn read_geojson() -> Result<GeoJson> {
    let file = File::open("../data/features_wingerworth.geojson")?;
    let reader = BufReader::new(file);
    let gj: GeoJson = serde_json::from_reader(reader)?;
    Ok(gj)
}