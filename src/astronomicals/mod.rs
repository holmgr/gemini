use nalgebra::geometry::Point3 as Point;

pub mod star;
pub mod planet;
pub mod system;
pub mod sector;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
/// Main galaxy containing all systems.
pub struct Galaxy {
    pub sectors: Vec<sector::Sector>,
}

impl Galaxy {
    pub fn new(sectors: Vec<sector::Sector>) -> Self {
        Galaxy { sectors }
    }
}

/// Hash based on location, algorithm used is presented in the paper:
/// Optimized Spatial Hashing for Collision Detection of Deformable Objects.
pub fn hash(location: Point<f64>) -> u64 {
    let values = location
        .iter()
        .zip(&[73856093f64, 19349663f64, 83492791f64])
        .map(|(&a, &b)| (a * b) as u64)
        .collect::<Vec<_>>();
    values.iter().fold(0, |acc, &val| acc ^ val)
}
