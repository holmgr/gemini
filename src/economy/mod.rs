use std::fmt;

/// A tradable and possibly producable commodity
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Commodity {
    Chemical,
    ConsumerItem,
    Food,
    IllegalDrug,
    IndustrialMaterial,
    LegalDrug,
    Machinery,
    Medicine,
    Metal,
    Mineral,
    Salvage,
    Slavery,
    Technology,
    Textile,
    Waste,
    Weapon,
}

impl fmt::Display for Commodity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Commodity::Chemical => "Chemicals",
                Commodity::ConsumerItem => "Consumer Items",
                Commodity::Food => "Food",
                Commodity::IllegalDrug => "Illegal Drugs",
                Commodity::IndustrialMaterial => "Industrial Materials",
                Commodity::LegalDrug => "Legal Drugs",
                Commodity::Machinery => "Machinery",
                Commodity::Medicine => "Medicine",
                Commodity::Metal => "Metals",
                Commodity::Mineral => "Minerals",
                Commodity::Salvage => "Salvage",
                Commodity::Slavery => "Slaves",
                Commodity::Technology => "Technology",
                Commodity::Textile => "Textiles",
                Commodity::Waste => "Waste",
                Commodity::Weapon => "Weapons",
            }
        )
    }
}
