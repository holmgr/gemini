use nalgebra::geometry::Point3 as Point;
use astronomicals::star::Star;
use astronomicals::planet::Planet;

#[derive(Debug, Builder, Clone)]
/// Represets a single star system with at a given location with the given
/// star and planets.
pub struct System {
    pub location: Point<f64>,
    pub name: String,
    pub star: Star,
    pub satelites: Vec<Planet>,
}
