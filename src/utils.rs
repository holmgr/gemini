use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

use nalgebra::geometry::Point2;
use spade::PointN;

/// Alias for 3D Point from nalgebra.
pub type Point = Point2<f64>;

/// Point with weight associated so that it can be ordered.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrdPoint {
    pub point: Point,
    pub weight: u32,
}

impl OrdPoint {
    pub fn new(point: Point, weight: u32) -> Self {
        OrdPoint { point, weight }
    }
}

impl Ord for OrdPoint {
    fn cmp(&self, other: &OrdPoint) -> Ordering {
        other.weight.partial_cmp(&self.weight).unwrap()
    }
}

impl PartialOrd for OrdPoint {
    fn partial_cmp(&self, other: &OrdPoint) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrdPoint {
    fn eq(&self, other: &OrdPoint) -> bool {
        self.weight == other.weight
    }
}

impl Eq for OrdPoint {}

/// Wrapper type implementing hashing for Point etc.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct HashablePoint(Point);

impl HashablePoint {
    pub fn new(point: Point) -> HashablePoint {
        HashablePoint { 0: point }
    }
    pub fn as_point(&self) -> &Point {
        &self.0
    }
}

impl Hash for HashablePoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .iter()
            .zip(&[73856093f64, 19349663f64])
            .map(|(&a, &b)| (a * b) as u64)
            .fold(0, |acc, val| acc ^ val)
            .hash(state);
    }
}

impl Eq for HashablePoint {}

impl PointN for HashablePoint {
    type Scalar = f64;

    fn dimensions() -> usize {
        2
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        &(self.0)[index]
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        &mut (self.0)[index]
    }

    fn from_value(value: Self::Scalar) -> HashablePoint {
        HashablePoint {
            0: Point::new(value, value),
        }
    }
}
