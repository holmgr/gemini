use game::Updatable;
use utils::{edit_distance, HashablePoint, OrdPoint, Point};

pub mod galaxy;
pub mod planet;
pub mod sector;
pub mod star;
pub mod system;

// Useful shorthand imports.
pub use self::galaxy::Galaxy;
pub use self::planet::Planet;
pub use self::sector::Sector;
pub use self::star::Star;
pub use self::system::System;

/// Hash based on location, algorithm used is presented in the paper:
/// Optimized Spatial Hashing for Collision Detection of Deformable Objects.
pub fn hash(location: &Point) -> u64 {
    let values = location
        .iter()
        .zip(&[73_856_093f64, 19_349_663f64, 83_492_791f64])
        .map(|(&a, &b)| (a * b) as u64)
        .collect::<Vec<_>>();
    values.iter().fold(0, |acc, &val| acc ^ val)
}
