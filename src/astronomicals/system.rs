use nalgebra::geometry::Point3 as Point;
use astronomicals::star::Star;
use astronomicals::planet::Planet;

#[derive(Debug, Builder)]
pub struct System {
    pub location: Point<f64>,
    pub name: String,
    pub star: Star,
    pub satelites: Vec<Planet>,
}

impl System {
    pub fn new(location: Point<f64>, name: String, star: Star, satelites: Vec<Planet>) -> Self {
        System {
            location,
            name,
            star,
            satelites,
        }
    }
}
