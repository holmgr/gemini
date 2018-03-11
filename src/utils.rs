use std::hash::{Hash, Hasher};

use nalgebra::geometry::Point3;
use spade::PointN;

/// Alias for 3D Point from nalgebra.
pub type Point = Point3<f64>;

/// Wrapper type implementing hashing for Point etc.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct HashablePoint(Point);

impl HashablePoint {
    pub fn as_point(&self) -> &Point {
        &self.0
    }
}

impl Hash for HashablePoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .iter()
            .zip(&[73856093f64, 19349663f64, 83492791f64])
            .map(|(&a, &b)| (a * b) as u64)
            .fold(0, |acc, val| acc ^ val)
            .hash(state);
    }
}

impl PointN for HashablePoint {
    type Scalar = f64;

    fn dimensions() -> usize {
        3
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        &(self.0)[index]
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        &mut (self.0)[index]
    }

    fn from_value(value: Self::Scalar) -> HashablePoint {
        HashablePoint {
            0: Point::new(value, value, value),
        }
    }
}
