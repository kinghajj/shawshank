use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

use num::{Bounded, ToPrimitive, FromPrimitive};
use owning_ref::StableAddress;

use prison::{Error, Prison, Solitary};

/// Flexible builder for [`Prison`].
///
/// ```
/// use std::sync::Arc;
///
/// let b1 = shawshank::Builder::<String>::new();
/// let mut p1 = b1.hash().unwrap();
/// assert_eq!(p1.intern("hello"), Ok(0));
/// assert_eq!(p1.resolve(0), Ok("hello"));
///
/// let b2 = shawshank::Builder::<Arc<String>>::new();
/// let mut p2 = b2.solitary_hash().unwrap();
/// assert_eq!(p2.intern("hello"), Ok(0));
/// let s: &String = p2.resolve(0).unwrap();
/// assert_eq!(s.as_str(), "hello");
/// ```
///
/// [`Prison`]: struct.Prison.html
pub struct Builder<O, I = usize> {
    _o: PhantomData<O>,
    _i: PhantomData<I>,
}

impl<O, I> Builder<O, I> {
    pub fn new() -> Self {
        Builder {
            _o: PhantomData,
            _i: PhantomData,
        }
    }
}

/// Create a [`Builder`] where the ID is `usize`.
/// [`Builder`]: struct.Builder.html
pub fn builder<O>() -> Builder<O> {
    Builder::<O>::new()
}

impl<O, I> Builder<O, I>
where O: StableAddress,
      I: Bounded + ToPrimitive + FromPrimitive
{
    /// Create an empty [`Prison`] that uses a `HashMap`.
    /// [`Prison`]: struct.Prison.html
    pub fn hash(&self) -> Result<Prison<O, I, HashMap<&'static O::Target, I>>, Error>
        where O::Target: Eq + Hash {
        Prison::new()
    }

    /// Create an empty [`Prison`] that uses a `BTreeMap`.
    /// [`Prison`]: struct.Prison.html
    pub fn btree(&self) -> Result<Prison<O, I, BTreeMap<&'static O::Target, I>>, Error>
        where O::Target: Eq + Ord {
        Prison::new()
    }
}

impl<O, I> Builder<O, I>
where O: StableAddress,
      O::Target: 'static + StableAddress,
      < O::Target as Deref >::Target: 'static,
      I: Bounded + ToPrimitive + FromPrimitive
{
    /// Create an empty [`Solitary`] that uses a `HashMap`.
    /// [`Solitary`]: struct.Solitary.html
    pub fn solitary_hash(&self) -> Result<Solitary<O, O::Target, I, HashMap<&'static < O::Target as Deref >::Target, I>>, Error>
        where < O::Target as Deref >::Target: Eq + Hash {
        Prison::new().map(|p| Solitary(p))
    }

    /// Create an empty [`Solitary`] that uses a `BTreeMap`.
    /// [`Solitary`]: struct.Solitary.html
    pub fn solitary_btree(&self) -> Result<Solitary<O, O::Target, I, BTreeMap<&'static < O::Target as Deref >::Target, I>>, Error>
        where < O::Target as Deref >::Target: Eq + Ord {
        Prison::new().map(|p| Solitary(p))
    }
}