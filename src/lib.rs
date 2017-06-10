//! [`ArenaSet`] is an efficient, generic internment structure.
//!
//! [`ArenaSet`]: struct.ArenaSet.html

#![cfg_attr(feature = "unstable", feature(test))]

extern crate num_traits;
extern crate owning_ref;

#[cfg(test)]
extern crate rand;

#[cfg(all(feature = "unstable", test))]
extern crate test;

mod arena_set;
mod builder;
mod traits;
mod utility;
#[macro_use] mod macros;

#[cfg(all(feature = "unstable", test))]
mod benches;

pub use builder::{Builder, builder};
pub use arena_set::{Error, ArenaSet, StadiumSet};
pub use traits::Map;
pub use utility::{string_arena_set, byte_arena_set, string_stadium_set, byte_stadium_set};
