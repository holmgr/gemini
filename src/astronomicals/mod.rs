use game::Updatable;
use utils::{edit_distance, OrdPoint, Point};

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
