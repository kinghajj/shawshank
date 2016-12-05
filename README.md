shawshank
=========

An efficient, generic internment structure.

[![Travis-CI Status](https://travis-ci.org/kinghajj/shawshank.png?branch=master)](https://travis-ci.org/kinghajj/shawshank)
[![Crates.io](https://img.shields.io/crates/v/shawshank.svg?maxAge=2592000)](https://crates.io/crates/shawshank)
[![License](https://img.shields.io/crates/l/shawshank.svg)](LICENSE)

```rust
extern crate shawshank;

fn main() {
    // prototypical motivation: string internment
    let mut sp = shawshank::string_arena_set();
    assert_eq!(sp.intern("hello"), Ok(0));
    assert_eq!(sp.intern("world"), Ok(1));
    assert_eq!(sp.intern("hello"), Ok(0));
    assert_eq!(sp.resolve(1), Ok("world"));

    // byte vectors work, too
    let mut bp = shawshank::byte_arena_set();
    assert_eq!(bp.intern(&[0, 1, 2][..]), Ok(0));

    // even Box<T>
    let mut p = shawshank::builder::<Box<u8>>().hash().unwrap();
    assert_eq!(p.intern(255), Ok(0));

    // BTreeMap instead of default HashMap
    let mut bsp = shawshank::builder::<String>().btree().unwrap();
    assert_eq!(bsp.intern("foo"), Ok(0));
}
```

For more details, see the [docs].

[docs]: https://kinghajj.github.io/shawshank/shawshank/index.html
