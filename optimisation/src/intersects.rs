use geo::{Intersects, Polygon};
use trenching_optimisation::TrenchLayout;

pub fn test(polygon_a: &Polygon<f64>, trenches: &TrenchLayout) -> bool {
    match trenches {
        TrenchLayout::CentreLine(trenches) => {
            if polygon_a.intersects(trenches) {
                return true;
            } else {
                return false;
            }
        }
        TrenchLayout::Continuous(trenches) => {
            if polygon_a.intersects(trenches) {
                return true;
            } else {
                return false;
            }
        }
        _ => {
            panic!("Trench pattern not yet added");
        }
    }
}
