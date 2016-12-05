use std::sync::Arc;

use builder::builder;
use arena_set::{ArenaSet, StadiumSet};

/// Create an [`ArenaSet`] for `String` with a `HashMap` and an ID of `usize`.
/// [`ArenaSet`]: struct.ArenaSet.html
pub fn string_arena_set() -> ArenaSet<String> {
    builder().hash().unwrap()
}

/// Create an [`ArenaSet`] for `Vec<u8>` with a `HashMap` and an ID of `usize`.
/// [`ArenaSet`]: struct.ArenaSet.html
pub fn byte_arena_set() -> ArenaSet<Vec<u8>> {
    builder().hash().unwrap()
}

/// Create a [`StadiumSet`] for `Arc<String>` with a `HashMap` and an ID of `usize`.
/// [`StadiumSet`]: struct.StadiumSet.html
pub fn string_stadium_set() -> StadiumSet<Arc<String>> {
    builder().stadium_set_hash().unwrap()
}

/// Create a [`StadiumSet`] for `Arc<Vec<u8>>` with a `HashMap` and an ID of `usize`.
/// [`StadiumSet`]: struct.StadiumSet.html
pub fn byte_stadium_set() -> StadiumSet<Arc<Vec<u8>>> {
    builder().stadium_set_hash().unwrap()
}
