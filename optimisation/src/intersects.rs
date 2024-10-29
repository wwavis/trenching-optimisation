use geo::{Intersects, LineString, Polygon};
// use geo::line_string;

// pub fn test(polygon_a: Polygon<f64>, polygon_b: Polygon<f64>) {
pub fn test(polygon_a: Polygon<f64>) -> bool{

    // println!("Testing intersects");

    // let polygon_a = Polygon::new(
    //     LineString::from(vec![(0., 0.), (0., 10.), (10., 10.), (10., 0.), (0., 0.)]),
    //     vec![],
    // );
    // let polygon_b = Polygon::new(
    //     LineString::from(vec![(0.5, 0.5), (1.5, 1.5), (1.5, 0.5), (0.5, 0.5)]),
    //     vec![],
    // );

    let polygon_b = Polygon::new(
        LineString::from(vec![(505600.0, 175700.5), (506600.0, 175700.5), (506600.0, 176700.5), (505600.0, 176700.5), (505600.0, 175700.5)]),
        vec![],
    );
  
    // assert!(polygon_a.intersects(&polygon_b));
    if polygon_a.intersects(&polygon_b) {
        return true;
    } else {
        return false;
    }

}