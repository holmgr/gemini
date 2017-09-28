#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate preferences;
extern crate notify;
extern crate rand;
extern crate petgraph;

mod game_config;
mod markov;

fn main() {

    // Load GameConfig from disk
    let mut config = game_config::GameConfig::retrieve();
    println!("Initial config is: {:?}", config);

    // Reload GameConfig if file on disk changes
    loop {
        match game_config::GameConfig::await_update() {
            Ok(new_config) => {
                config = new_config;
                println!("Updated, config now: {:?}", config);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
