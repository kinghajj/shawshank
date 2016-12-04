use std::sync::Arc;

use builder::builder;
use prison::{Prison, Solitary};

/// Create a [`Prison`] for `String` with a `HashMap` and an ID of `usize`.
/// [`Prison`]: struct.Prison.html
pub fn string_prison() -> Prison<String> {
    builder().hash().unwrap()
}

/// Create a [`Prison`] for `Vec<u8>` with a `HashMap` and an ID of `usize`.
/// [`Prison`]: struct.Prison.html
pub fn byte_prison() -> Prison<Vec<u8>> {
    builder().hash().unwrap()
}

/// Create a [`Solitary`] for `Arc<String>` with a `HashMap` and an ID of `usize`.
/// [`Solitary`]: struct.Solitary.html
pub fn string_solitary() -> Solitary<Arc<String>> {
    builder().solitary_hash().unwrap()
}

/// Create a [`Solitary`] for `Arc<Vec<u8>>` with a `HashMap` and an ID of `usize`.
/// [`Solitary`]: struct.Solitary.html
pub fn byte_solitary() -> Solitary<Arc<Vec<u8>>> {
    builder().solitary_hash().unwrap()
}
