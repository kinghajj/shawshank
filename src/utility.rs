use std::sync::Arc;

use builder::builder;
use arena_set::{ArenaSet, StatiumSet};

/// Create a [`ArenaSet`] for `String` with a `HashMap` and an ID of `usize`.
/// [`ArenaSet`]: struct.ArenaSet.html
pub fn string_arena_set() -> ArenaSet<String> {
    builder().hash().unwrap()
}

/// Create a [`ArenaSet`] for `Vec<u8>` with a `HashMap` and an ID of `usize`.
/// [`ArenaSet`]: struct.ArenaSet.html
pub fn byte_arena_set() -> ArenaSet<Vec<u8>> {
    builder().hash().unwrap()
}

/// Create a [`StatiumSet`] for `Arc<String>` with a `HashMap` and an ID of `usize`.
/// [`StatiumSet`]: struct.StatiumSet.html
pub fn string_stadium_set() -> StatiumSet<Arc<String>> {
    builder().stadium_set_hash().unwrap()
}

/// Create a [`StatiumSet`] for `Arc<Vec<u8>>` with a `HashMap` and an ID of `usize`.
/// [`StatiumSet`]: struct.StatiumSet.html
pub fn byte_stadium_set() -> StatiumSet<Arc<Vec<u8>>> {
    builder().stadium_set_hash().unwrap()
}
