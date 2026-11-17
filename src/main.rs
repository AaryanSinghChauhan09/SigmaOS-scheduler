//! scheduler component binary.
//!
//! Component logic lives in the library crate (`src/lib.rs`); this binary is
//! a thin entry point kept minimal so the crate builds and tests run.

fn main() {
    println!("SigmaOS {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
