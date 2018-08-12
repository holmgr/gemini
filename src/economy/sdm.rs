use super::*;
use std::{collections::HashMap, iter::FromIterator};

use utils::Point;

#[derive(Serialize, Deserialize)]
/// A multidimensional height map storing prices for each commodity at each location.
pub struct Sdm {
    rbf: RBF,
    map: HashMap<Point, HashMap<Commodity, f64>>,
}

impl Sdm {
    pub fn new(rbf: RBF) -> Self {
        Sdm {
            rbf,
            map: HashMap::new(),
        }
    }

    pub fn add_point(&mut self, point: Point) {
        // Create heightmap with balanced suply/demand.
        self.map.insert(
            point,
            HashMap::from_iter(Commodity::values().map(|commodity| (*commodity, 0.))),
        );
    }

    pub fn reset(&mut self) {
        self.map.values_mut().for_each(|rhs| {
            *rhs = HashMap::from_iter(Commodity::values().map(|commodity| (*commodity, 0.)))
        });
    }

    pub fn update(&mut self, source: Point, export_prices: &[(Commodity, f64)]) {
        for (point, existing_prices) in self.map.iter_mut() {
            let distance_trans = self.rbf.interpolate(source.distance(point));
            for (commodity, export_price) in export_prices {
                let existing_price = existing_prices.get_mut(&commodity).unwrap();
                *existing_price = existing_price.min(distance_trans * export_price);
            }
        }
    }

    pub fn map<'a>(&'a self) -> impl Iterator<Item = (&'a Point, &'a HashMap<Commodity, f64>)> {
        self.map.iter()
    }
}

#[derive(Serialize, Deserialize)]
pub enum RBF {
    Quadratic,
}

impl RBF {
    pub fn interpolate(&self, radius: f64) -> f64 {
        /// TODO: Tune the min value.
        let min = 0.7;
        match self {
            RBF::Quadratic => radius.powi(2) + min
        }
    }
}
