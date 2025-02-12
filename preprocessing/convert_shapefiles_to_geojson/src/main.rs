use shapefile_to_geojson::convert_shapefile_to_geojson;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let args: Vec<String> = std::env::args().collect();
    convert_shapefile_to_geojson(&args[1], &args[2]).await?;

    Ok(())
}
