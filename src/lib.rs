//! [`Prison`] is an efficient, generic internment structure.
//!
//! [`Prison`]: struct.Prison.html

#![cfg_attr(feature = "unstable", feature(test))]

extern crate num;
extern crate owning_ref;

#[cfg(test)]
extern crate rand;

#[cfg(all(feature = "unstable", test))]
extern crate test;

mod builder;
mod prison;
mod traits;
mod utility;
#[macro_use] mod macros;

#[cfg(all(feature = "unstable", test))]
mod benches;

pub use builder::{Builder, builder};
pub use prison::{Error, Prison, Solitary};
pub use traits::Map;
pub use utility::{string_prison, byte_prison, string_solitary, byte_solitary};
