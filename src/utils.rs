use std::{cmp::{min, Ordering},
          hash::{Hash, Hasher},
          mem::swap};

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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HashablePoint(Point);

impl HashablePoint {
    pub fn new(point: Point) -> HashablePoint {
        HashablePoint { 0: point }
    }
    pub fn as_point(&self) -> &Point {
        &self.0
    }
}

impl PartialEq for HashablePoint {
    fn eq(&self, other: &HashablePoint) -> bool {
        self.0 == other.0
    }
}

impl Hash for HashablePoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .iter()
            .zip(&[73_856_093f64, 19_349_663f64])
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

/// Returns the edit distance between strings `a` and `b` using Levenshtein
/// distance.
/// The runtime complexity is `O(m*n)`, where `m` and `n` are the
/// strings' lengths.
pub fn edit_distance(a: &str, b: &str) -> i32 {
    // Handle zero length case.
    if a.is_empty() {
        return b.chars().count() as i32;
    } else if b.is_empty() {
        return a.chars().count() as i32;
    }

    let len_b = b.chars().count() + 1;

    let mut pre = vec![0; len_b];
    let mut cur = vec![0; len_b];

    // Initialize string b.
    for (i, prev) in pre.iter_mut().enumerate().take(len_b).skip(1) {
        *prev = i as i32;
    }

    // Calculate edit distance.
    for (i, ca) in a.chars().enumerate() {
        // Get first column for this row.
        cur[0] = (i as i32) + 1;
        for (j, cb) in b.chars().enumerate() {
            cur[j + 1] = min(
                // Deletion.
                pre[j + 1] + 1,
                min(
                    // Insertion.
                    cur[j] + 1,
                    // Match or substitution.
                    pre[j] + if ca == cb { 0 } else { 1 },
                ),
            );
        }
        swap(&mut cur, &mut pre);
    }

    pre[len_b - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("foo", "foobar"), 3);
        assert_eq!(edit_distance("foo", "bar"), 3);
        assert_eq!(edit_distance("bar", "baz"), 1);
    }
}
