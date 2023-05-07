# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unrelease] - yyyy-mm-dd

### Added

- Configure a simulation with `Simulation::Config` and `Simulation::from_config` to reduce
  boilerplate. `Simulation::Config` are cloneable, and support specifying simulation seeds.

### Changed

- BREAKING: Changed public exports on some of the more hidden internals of `Simulation`'s
- BREAKING: Changed `Simulation::new()` construction
- Split the counting/aggreagting of the simulation data into it's own crate

### Fixed

## [0.1.0] - 2020-04-30

First Release, heavy WIP, not ready for real use.

