use rand::Rng;
use statrs::distribution::{Continuous, Distribution, Exponential, Gamma};
use std::f64::consts::PI;

use astronomicals::{planet::{PlanetBuilder, PlanetEconomy, PlanetType}, star::Star};

/// Basic non deterministic name generator for generating new Planets which
/// are similar to the trained data provided.
pub struct PlanetGen {
    mass_gen: Exponential,
    orbit_dist_gen: Gamma,
}

impl PlanetGen {
    /// Minimal distance to star.
    // TODO: Move to config.
    const MIN_DIST: f64 = 500.;

    /// Create a new Planet generator which loads the star resources needed.
    pub fn new() -> Self {
        let mass_gen = Exponential::new(1. / 3.).unwrap();
        let orbit_dist_gen = Gamma::new(0.28, 0.17).unwrap();
        PlanetGen {
            mass_gen,
            orbit_dist_gen,
        }
    }

    /// Calculates the initial planet population based on mass and planet type.
    pub fn initial_population(mass: f64, kind: &PlanetType) -> f64 {
        let mass_factor = Gamma::new(7., 5.).unwrap();
        let type_factor: f64 = match *kind {
            PlanetType::Metal => 150.,
            PlanetType::Earth => 800.0,
            PlanetType::Rocky => 1.,
            PlanetType::Icy => 0.5,
            PlanetType::GasGiant => 0.,
        };
        mass_factor.pdf(mass) * type_factor * 6.
    }

    /// Calculate planet surface temperature from star luminosity and distance
    /// to it. Uses the Bond albedo for the Earth.
    pub fn calculate_surface_temperature(orbit_distance: f64, star: &Star) -> f64 {
        (star.luminosity * 3.846 * 10f64.powi(26) * (1. - 0.29)
            / (16. * PI * (299_692_458. * orbit_distance).powi(2) * 5.670_373 * 10f64.powi(-8)))
            .powf(0.25)
    }

    /// Predict the planet type based on surface_temperature and mass.
    pub fn predict_type<R: Rng>(rng: &mut R, surface_temperature: f64, mass: f64) -> PlanetType {
        // Based on trained decision tree with modifications to allow for
        // Earth-like planets at a higher rate.
        let random_val: f64 = rng.gen();
        match (surface_temperature, mass) {
            (_, y) if y >= 5.185 => PlanetType::GasGiant,
            (x, y) if x < 124.5 && y < 5.185 => PlanetType::Icy,
            (x, _) if x > 280. && x < 310. => PlanetType::Earth,
            (x, _) if x >= 124.5 && random_val < 0.8 => PlanetType::Rocky,
            (x, _) if x >= 124.5 && random_val >= 0.8 => PlanetType::Metal,
            _ => PlanetType::Rocky,
        }
    }

    /// Predict the planet economy type based on type.
    pub fn predict_economy<R: Rng>(rng: &mut R, kind: &PlanetType) -> PlanetEconomy {
        // Based on trained decision tree with modifications to allow for
        // Earth-like planets at a higher rate.
        let random_val: f64 = rng.gen();
        match *kind {
            PlanetType::Icy if random_val < 0.2 => PlanetEconomy::Extraction,
            PlanetType::Icy if random_val < 0.4 => PlanetEconomy::Refinary,
            PlanetType::Icy if random_val < 0.7 => PlanetEconomy::HighTech,
            PlanetType::Icy if random_val < 0.8 => PlanetEconomy::Military,
            PlanetType::Icy => PlanetEconomy::Industrial,
            PlanetType::Rocky if random_val < 0.2 => PlanetEconomy::Extraction,
            PlanetType::Rocky if random_val < 0.4 => PlanetEconomy::Refinary,
            PlanetType::Rocky if random_val < 0.7 => PlanetEconomy::HighTech,
            PlanetType::Rocky if random_val < 0.9 => PlanetEconomy::Military,
            PlanetType::Rocky => PlanetEconomy::Industrial,
            PlanetType::Earth => PlanetEconomy::Agriculture,
            PlanetType::Metal => PlanetEconomy::Extraction,
            _ => PlanetEconomy::None,
        }
    }

    /// Generates a new PlanetBuilder from the _distribution_ using the provided random
    /// generator. Sets the fields which are independent on the context.
    pub fn generate<R: Rng>(&self, gen: &mut R) -> Option<PlanetBuilder> {
        let mass = self.mass_gen.sample(gen);

        // Magic constant, needed to scale back since scaling needed to fit gamma.
        let orbit_distance = PlanetGen::MIN_DIST + 1000. * self.orbit_dist_gen.sample(gen);

        // TODO: Make something a bit more accurate regarding planet type and gravity.
        Some(
            PlanetBuilder::default()
                .mass(mass)
                .orbit_distance(orbit_distance)
                .gravity(mass)
                .to_owned(),
        )
    }
}
