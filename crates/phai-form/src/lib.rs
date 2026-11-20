//! The `.phaiform` file (docs/phaiform.md): read, check, write, generate Rust from it, build it live.

pub mod arrange;
pub mod bars;
pub mod codegen;
pub mod events;
pub mod live;
pub mod model;
pub mod props;
pub mod report;
pub mod web;

pub use model::*;
pub use toml;
