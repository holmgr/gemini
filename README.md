# Gemini
![](https://travis-ci.org/holmgr/gemini.svg?branch=master)
[![codecov](https://codecov.io/gh/holmgr/gemini/branch/master/graph/badge.svg)](https://codecov.io/gh/holmgr/gemini)

Science fiction space trading/smuggling simulation game using procedural generation.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

Source code documentation for the project is available at: https://holmgr.github.io/gemini/ which is automatically built from master on every update.

### Prerequisites

To start developing or running the project you will need to install Rust stable, preferably using the Rustup tool

```
rustup install stable
```

### Installing

Start by cloning the repository, then installing all the dependencies is as simple as running the following in the root directory:

```
cargo build
```

To run the project execute the following:

```
cargo run
```

Since the project uses quite a lot of compuations for generating a new galaxy you might want to run with optimizations on.
To do that run:

```
cargo run --release
```

Furthermore, to see debug and info logs you need to run with the RUST_LOG environment variable:

```
bash/zsh: RUST_LOG=gemini=debug cargo run
fish: env RUST_LOG=gemini=debug cargo run
```

To build and install the executable on your system simply run:

```
cargo install
```

## Running the tests

To run the automated tests, run:

```
cargo test
```

Note: No branch or pull request can be merged before all tests has passed in Travis.

### And coding style tests

This project follows the Rust standard as specified by the Rustfmt project.
Install Rustfmt by doing the following:

```
rustup component add rustfmt-preview --toolchain=nightly
```

To format the code run:

```
cargo +nightly fmt
```

Note: No branch or pull request can be merged before the style-guide has passed Rustfmt.

## Built With

* [Rust](https://www.rust-lang.org/en-US/) - The Rust language

## Contributing

Taking no pull requests or issues as of this moment as the project is very much in early stages of development.

## Authors

* **Viktor Holmgren** - [holmgr](https://github.com/holmgr)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments

* Inspiration is drawn from the fantastic games: Elite Dangerous and Dwarf Fortress

