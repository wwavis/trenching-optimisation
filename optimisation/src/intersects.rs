use geo::{Intersects, Polygon};
use trenching_optimisation::TrenchLayout;

pub fn test(feature: &Polygon<f64>, trenches: &TrenchLayout) -> bool {
    feature.intersects(&trenches.0)
}
