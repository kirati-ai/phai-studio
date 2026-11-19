//! The `.phaiform` file (docs/phaiform.md): read, check, write, generate Rust from it, build it live.

pub mod arrange;
pub mod bars;
pub mod codegen;
pub mod live;
pub mod model;
pub mod props;

pub use model::*;
pub use toml;
