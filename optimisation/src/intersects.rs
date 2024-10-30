use geo::{Intersects, Polygon};
use trenching_optimisation::TrenchPattern;
// use geo::line_string;

pub fn test(polygon_a: Polygon<f64>, trenches: &TrenchPattern) -> bool {
    match trenches {
        TrenchPattern::CentreLine(trenches) => {
            if polygon_a.intersects(trenches) {
                return true;
            } else {
                return false;
            }
        }
        _ => {
            println!("Trench pattern yet not added");
            return false;
        },
    }
}
