mod names;

trait Gen {
   type GenItem; 
   type TrainData;

   fn new(seed: u32) -> Self;
   fn train(&mut self, &Self::TrainData);
   fn generate(&mut self) -> Option<Self::GenItem>;
}
