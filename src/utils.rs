use spade::{PointN, TwoDimensional};
use std::{cmp::{min, Ordering},
          hash::{Hash, Hasher},
          mem::swap,
          ops::{Add, AddAssign, MulAssign}};

/// Generic Point type for geometry.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point.
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    /// Create a new point with origin coordinates.
    pub fn origin() -> Point {
        Point::new(0., 0.)
    }

    /// Returns the euclidian distance to another point.
    pub fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Returns the hash of the point coordinates.
    /// Hash based on algorithm used is presented in the paper:
    /// Optimized Spatial Hashing for Collision Detection of Deformable Objects.
    pub fn hash(&self) -> u64 {
        ((self.x * 73_856_093f64) as u64 ^ (self.y * 19_349_663f64) as u64)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl MulAssign<f64> for Point {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash().hash(state);
    }
}

impl Eq for Point {}

impl TwoDimensional for Point {}

impl PointN for Point {
    type Scalar = f64;

    fn dimensions() -> usize {
        2
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        match index {
            0 => &self.x,
            _ => &self.y,
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            _ => &mut self.y,
        }
    }

    fn from_value(value: Self::Scalar) -> Point {
        Point::new(value, value)
    }
}

/// Point with weight associated so that it can be ordered.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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
