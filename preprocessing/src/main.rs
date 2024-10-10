use shapefile;

fn main() {
    println!("Preprocessing");

    let filename = "../../real_sitedata/Wingerworth/features_wingerworth.shp";
    let mut reader = shapefile::Reader::from_path(filename).unwrap();

    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result.unwrap();
        println ! ("Shape: {}, records: ", shape);
        for (name, value) in record {
            println ! ("\t{}: {:?}, ", name, value);
        }
        println ! ();
    }

}