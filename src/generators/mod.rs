pub mod names;

/// Generic Generator trait to be implemented by concrete generators of different kinds.
pub trait Gen {
    type GenItem;
    type TrainData;

    /// Create a new genrator with the given seed for the random generator
    fn new(seed: u32) -> Self;

    /// Train the generator with the given data
    fn train(&mut self, &Self::TrainData);

    /// Generate a new item from the generator, can be None if the generator is empty etc.
    fn generate(&mut self) -> Option<Self::GenItem>;
}
