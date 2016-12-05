use std::borrow::Borrow;
use std::collections::HashMap;
use std::mem;
use std::marker::PhantomData;
use std::ops::Deref;

use num::{Bounded, ToPrimitive, FromPrimitive};
use owning_ref::StableAddress;

use traits::Map;

/// An efficient, generic internment structure.
///
/// # Internals
///
/// When specialized for `String`/`str`, the structure looks like this:
///
/// ```
/// use std::collections::HashMap;
///
/// enum Slot<T> {
///     Vacant(usize),
///     Occupied(T),
/// }
/// pub struct StringArenaSet {
///     map: HashMap<&'static str, usize>,
///     interned: Vec<Slot<String>>,
///     head: usize,
/// }
/// ```
///
/// A simpler structure that can be expressed entirely in safe Rust might use
/// `HashMap<String, usize>`. The obvious drawback, however, is that each
/// interned string is stored twice. Since a slice stores a pointer to the heap
/// of a `String`, however, and not to that of the `Vec`, we can save space by
/// using them instead as keys. If we enforce the variant that slices in the
/// map are removed in lock-step with the strings in the vector, then it's safe
/// to lie to the compiler by changing the lifetime to `'static`.
///
/// This structure generalizes to all pairs of "owned" and "reference" types
/// where moving the "owned" doesn't invalidate the "reference." The
/// `owning_ref` crate provides the trait `StableAddress` to mark such types.
/// Other examples are `Vec<T>`/`[T]` and `Box<T>`/`T`, where `T: Clone`.
///
/// `head` contains the index of the first vacant slot, which in turn has the
/// index of the next, etc., effectively forming a linked list, with `!0` as the
/// end-of-list marker. This allows vacant slots to be efficiently reclaimed
/// before appending to the vector. This is the same technique employed by
/// [`vec_arena`].
///
/// # Custom ID Types
///
/// By default, the ID type parameter `I` is `usize`, the type of a `Vec`
/// index. One problem with `usize`, of course, is lack of type safety. One
/// could wrap the IDs inside tuple structs, so that different domains share the
/// same `ArenaSet`. Some workloads, however, may have domains with disjoint
/// sets or much lower cardinality than `usize` provides. In such cases,
/// more space can be saved by storing a smaller-than-`usize` `I` in the
/// internal map, and converting to and from `usize` as needed. [`intern`]
/// returns [`Error::IdOverflow`] if there are no more unique IDs available.
///
/// The `ToPrimitive`/`FromPrimitive` traits of the `num` crate are used to
/// perform the conversions. If these ever return `None` during an operation,
/// it will fail with [`Error::FromIdFailed`]/[`Error::ToIdFailed`].
///
/// The [`custom_intern_id!`] macro reduces the boilerplate to set thes up.
///
/// ```
/// #[macro_use] extern crate shawshank;
/// extern crate num;
///
/// use shawshank::Error;
///
/// // min/max optional; default to those of base type
/// custom_intern_id!(Small, u8, 0, 3);
///
/// fn main() {
///     let mut p = shawshank::Builder::<String, Small>::new().hash().unwrap();
///     assert_eq!(p.intern("one"), Ok(Small(0)));
///     assert_eq!(p.intern("two"), Ok(Small(1)));
///     assert_eq!(p.intern("three"), Ok(Small(2)));
///     assert_eq!(p.intern("four"), Ok(Small(3)));
///     assert_eq!(p.intern("fail"), Err(Error::IdOverflow));
///     assert_eq!(p.disintern(Small(0)), Ok("one".into()));
///     assert_eq!(p.intern("success"), Ok(Small(0)));
/// }
/// ```
///
/// # Type Parameters
///
///   * `O`: The "owened" type of interned items (e.g. `String`, `Vec<T>`).
///   * `I`: The "ID" type to uniquely resolve interned items.
///   * `M`: The type used to [`Map`] `O::Target`s to `I`s.
///
/// [`intern`]: struct.ArenaSet.html#method.intern
/// [`Error::FromIdFailed`]: enum.Error.html#variant.FromIdFailed
/// [`Error::ToIdFailed`]: enum.Error.html#variant.ToIdFailed
/// [`Error::IdOverflow`]: enum.Error.html#variant.IdOverflow
/// [`Map`]: trait.Map.html
/// [`custom_intern_id!`]: macro.custom_intern_id.html
/// [`vec_arena`]: https://github.com/stjepang/vec-arena
pub struct ArenaSet<O: StableAddress, I = usize, M = HashMap<&'static < O as Deref >::Target, I>> {
    map: M,
    interned: Vec<Slot<O>>,
    head: usize,
    max_idx: usize,
    _i: PhantomData<I>,
}

impl<O, I, M> ArenaSet<O, I, M>
where O: StableAddress,
      I: Bounded + ToPrimitive + FromPrimitive,
      M: Map {
    /// Create a new, empty ArenaSet.
    #[inline]
    pub fn new() -> Result<Self, Error> {
        Self::with_capacity(0)
    }

    /// Create a new, empty ArenaSet with a capacity hint.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Result<Self, Error> {
        Self::bounded_with_capacity(
            I::max_value().to_usize().ok_or(Error::FromIdFailed)? -
                I::min_value().to_usize().ok_or(Error::FromIdFailed)?,
            capacity)
    }

    /// Create a new, empty ArenaSet with a specific maximum index and a capacity hint.
    #[inline]
    pub fn bounded_with_capacity(max_idx: usize, capacity: usize) -> Result<Self, Error> {
        let max_possible = I::max_value().to_usize().ok_or(Error::FromIdFailed)?
            - I::min_value().to_usize().ok_or(Error::FromIdFailed)?;
        if max_idx > max_possible {
            return Err(Error::IdOverflow);
        }
        Ok(ArenaSet {
            map: M::with_capacity(capacity),
            max_idx: max_idx,
            head: !0,
            interned: Vec::with_capacity(capacity),
            _i: PhantomData,
        })
    }

    /// Get the number of interned items.
    ///
    /// ```
    /// let mut p = shawshank::string_arena_set();
    /// assert_eq!(p.intern("hello"), Ok(0));
    /// assert_eq!(p.intern("world"), Ok(1));
    /// assert_eq!(p.count(), 2);
    /// p.disintern(0).unwrap();
    /// assert_eq!(p.count(), 1);
    /// ```
    #[inline]
    pub fn count(&self) -> usize {
        self.map.len()
    }

    /// Get the capacity of the internal vector.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.interned.capacity()
    }

    /// Resolve in item by its unique ID.
    ///
    /// The success type is generic to both target and direct references.
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let mut p = shawshank::byte_stadium_set();
    /// assert_eq!(p.intern(vec![1,2,3]), Ok(0));
    /// let s1: &Vec<u8> = p.resolve(0).unwrap();
    /// let s2: &Arc<Vec<u8>> = p.resolve(0).unwrap();
    /// ```
    ///
    /// Complexity: _O(1)_
    #[inline]
    pub fn resolve<'a, U, Q: ? Sized>(&'a self, id: U) -> Result<&'a Q, Error>
        where U: Borrow<I>,
              O: Borrow<Q> {
        let ix = id.borrow().to_usize().ok_or(Error::FromIdFailed)?;
        let owned = self.interned.get(ix).ok_or(Error::InvalidId)?;
        match *owned {
            Slot::Occupied(ref item) => Ok(item.borrow()),
            _ => Err(Error::InvalidId)
        }
    }
}

// couldn't figure out how to get traits to abstract the differences
// between ArenaSet and StatiumSet, so had to resort to macros

macro_rules! insert {
    ( $this:ident, $item:ident, $to_owned:expr ) => { {
        // fast case: item already interned
        if let Some(entry) = $this.map.get(make_static($item.borrow())) {
            return Ok(*entry);
        }
        // don't let IDs overflow
        let cnt = $this.count();
        if cnt != 0 && cnt - 1 == $this.max_idx {
            return Err(Error::IdOverflow);
        }
        let owned = $to_owned($item);
        let reference = make_static(owned.deref());
        let ix =
        if $this.head == !0 {
            // invariant: no vacant slots
            $this.interned.push(Slot::Occupied(owned));
            $this.interned.len() - 1
        } else {
            // invariant: if `self.head != !0`, then it has an
            // index to vacant slot.
            let ix = $this.head;
            if let Slot::Vacant(next) = mem::replace(unsafe { $this.interned.get_unchecked_mut(ix) },
                                                     Slot::Occupied(owned)) {
                $this.head = next;
                ix
            } else {
                unreachable!()
            }
        };
        // convert to ID
        match I::from_usize(ix).ok_or(Error::ToIdFailed) {
            Ok(id) => {
                // complete internment
                $this.map.insert(reference, id);
                Ok(id)
            }
            Err(err) => {
                // revert internment.
                // invariant: something was just placed at `ix`,
                // so we don't need to check the access; and
                // `self.head` has been updated correctly.
                *unsafe { $this.interned.get_unchecked_mut(ix) } = Slot::Vacant($this.head);
                $this.head = ix;
                Err(err)
            }
        }
    } }
}

macro_rules! disintern {
    ( $this:expr, $id:ident) => { {
        let ix = $id.borrow().to_usize().ok_or(Error::FromIdFailed)?;
        match $this.interned.get_mut(ix) {
            None => Err(Error::InvalidId),
            Some(&mut Slot::Vacant(_)) => Err(Error::InvalidId),
            Some(occupied) => {
                // invariant: we just eliminated all other possibilities, so we know
                // it's occupied; and `self.head` has been updated correctly.
                if let Slot::Occupied(item) = mem::replace(occupied, Slot::Vacant($this.head)) {
                    $this.map.remove(make_static(item.deref()));
                    $this.head = ix;
                    Ok(item)
                } else {
                    unreachable!()
                }
            }
        }
    } }
}

macro_rules! shrink {
    ($this:ident, $t:ty) => { {
        let mut remap = <$t>::new();
        let mut shrunk = Vec::with_capacity($this.count());
        for (ix, oi) in $this.interned.drain(..).enumerate() {
            if let Slot::Occupied(i) = oi {
                let i_static = make_static(i.deref());
                match (I::from_usize(ix), I::from_usize(shrunk.len())) {
                    (Some(old_id), Some(new_id)) => {
                        remap.insert(old_id, new_id);
                        shrunk.push(Slot::Occupied(i))
                    }
                    _ => { $this.map.remove(i_static); }
                }
            }
        }
        $this.interned = shrunk;
        // invariant: no vacant slots
        $this.head = !0;
        $this.map.shrink_to_fit();
        remap
    } }
}

impl<O, I, M> ArenaSet<O, I, M>
where O: StableAddress,
      O::Target: 'static,
      I: Copy + ToPrimitive + FromPrimitive + Bounded,
      M: Map<Key = &'static O::Target, Value = I>
{
    /// Intern an item, receiving an ID that can later be used to [`resolve`] the original.
    ///
    /// If the item has already been interned, nothing changes, and the item's current ID
    /// is returned. Barring any calls to [`shrink`], this will be the same ID returned
    /// when the item was first interned.
    ///
    /// `item` is generic so that either a reference or owned type may be passed.
    ///
    /// ```
    /// let mut p = shawshank::string_arena_set();
    /// assert_eq!(p.intern("hello"), Ok(0));
    /// assert_eq!(p.intern(String::from("hello")), Ok(0));
    /// ```
    ///
    /// Complexity: _O(max([`M::get(K)`], [`M::insert(K,V)`]))_
    ///
    /// [`resolve`]: struct.ArenaSet.html#method.resolve
    /// [`shrink`]: struct.ArenaSet.html#method.shrink
    /// [`M::get(K)`]: trait.Map.html#tymethod.get
    /// [`M::insert(K,V)`]: trait.Map.html#tymethod.insert
    pub fn intern<Q>(&mut self, item: Q) -> Result<I, Error>
        where Q: Borrow<O::Target>,
              O: From<Q> {
        insert!(self, item, |item: Q| { O::from(item) })
    }

    /// Disintern an item by its unique ID.
    ///
    /// Barring any calls to [`shrink`], all subsequent calls to [`resolve`] with the ID
    /// will fail. If the item is interned again, it will get a different ID.
    ///
    /// ```
    /// let mut p = shawshank::string_arena_set();
    /// assert_eq!(p.intern("hello"), Ok(0));
    /// assert_eq!(p.intern("world"), Ok(1));
    /// assert_eq!(p.disintern(0), Ok("hello".into()));
    /// assert_eq!(p.resolve::<_, str>(0), Err(shawshank::Error::InvalidId));
    /// ```
    ///
    /// Complexity: _O([`M::remove(K)`])_
    ///
    /// [`resolve`]: struct.ArenaSet.html#method.resolve
    /// [`shrink`]: struct.ArenaSet.html#method.shrink
    /// [`M::remove(K)`]: trait.Map.html#tymethod.remove
    pub fn disintern<'a, U: Borrow<I>>(&'a mut self, id: U) -> Result<O, Error> {
        disintern!(self, id)
    }

    /// Shrink the internal data structures by re-using ID of disinterned items.
    /// Returns a map from the old IDs to the new ones.
    ///
    /// If an error occurs converting either the old or new index into a custom
    /// ID type, the item will be disinterned, an the resulting map will have no
    /// entry for the old ID.
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut p = shawshank::string_arena_set();
    /// assert_eq!(p.intern("hello"), Ok(0));
    /// assert_eq!(p.intern("world"), Ok(1));
    /// assert_eq!(p.disintern(0), Ok("hello".into()));
    /// let remap: BTreeMap<_, _> = p.shrink();
    /// assert_eq!(remap[&1], 0);
    /// assert_eq!(p.resolve(0), Ok("world"));
    /// ```
    ///
    /// Complexity: _O(successes * [`T::insert(K)`] + failures * [`M::remove(K)`])_
    ///
    /// [`T::insert(K)`]: trait.Map.html#tymethod.insert
    /// [`M::remove(K)`]: trait.Map.html#tymethod.remove
    pub fn shrink<T: Map<Key = I, Value = I>>(&mut self) -> T
    {
        shrink!(self, T)
    }
}

/// Specialization of [`ArenaSet`] where `O::Target: StableAddress`.
///
/// Example: if `O = Arc<Vec<u8>>`, then `O::Target = Vec<u8>`. Therefore,
/// the map `M` can be `HashMap<&'static u8, usize>`, rather than the
/// `HashMap<&'static Vec<u8>, usize>` that [`ArenaSet`] would use.
/// [`intern`] can similarly accept `&'a [u8]` instead of `&'a Vec<u8>`.
///
/// [`ArenaSet`]: struct.ArenaSet.html
/// [`intern`]: struct.StatiumSet.html#method.intern
pub struct StatiumSet<O: StableAddress<Target = R>, R: ? Sized + StableAddress = < O as Deref >::Target, I = usize, M = HashMap<&'static < R as Deref >::Target, I>>(pub ArenaSet<O, I, M>);

impl<O, R, I, M> StatiumSet<O, R, I, M>
where O: StableAddress<Target = R>,
      R: 'static + StableAddress,
      I: Copy + ToPrimitive + FromPrimitive + Bounded,
      M: Map<Key = &'static < R as Deref >::Target, Value = I>
{
    /// Analogue of [`intern`].
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use std::sync::Arc;
    ///
    /// let mut p = shawshank::byte_stadium_set();
    /// assert_eq!(p.intern(&[1,2,3][..]), Ok(0));
    /// assert_eq!(p.intern(vec![1,2,3]), Ok(0));
    /// ```
    ///
    /// [`intern`]: struct.ArenaSet.html#method.intern
    pub fn intern<Q>(&mut self, item: Q) -> Result<I, Error>
        where Q: Borrow<< O::Target as Deref >::Target>,
              O::Target: From<Q>,
              O: From<< O as Deref >::Target> {
        let ref mut this = self.0;
        insert!(this, item, |item: Q| { O::from(O::Target::from(item)) })
    }

    /// Analogue of [`disintern`].
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use std::ops::Deref;
    /// use std::sync::Arc;
    ///
    /// let mut p = shawshank::byte_stadium_set();
    /// assert_eq!(p.intern(&[1,2,3][..]), Ok(0));
    /// assert_eq!(p.disintern(0).unwrap().deref().deref(), &[1,2,3]);
    /// ```
    ///
    /// [`disintern`]: struct.ArenaSet.html#method.disintern
    pub fn disintern<'a, U: Borrow<I>>(&'a mut self, id: U) -> Result<O, Error> {
        let ref mut this = self.0;
        disintern!(this, id)
    }

    /// Analogue of [`resolve`].
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use std::sync::Arc;
    ///
    /// let mut p = shawshank::byte_stadium_set();
    /// assert_eq!(p.intern(&[1,2,3][..]), Ok(0));
    /// let s1: &Vec<u8> = p.resolve(0).unwrap();
    /// let s1: &Arc<Vec<u8>> = p.resolve(0).unwrap();
    /// ```
    ///
    /// [`resolve`]: struct.ArenaSet.html#method.resolve
    #[inline]
    pub fn resolve<'a, U, Q: ? Sized>(&'a self, id: U) -> Result<&'a Q, Error>
        where U: Borrow<I>,
              O: Borrow<Q> {
        self.0.resolve(id)
    }

    /// Analogue of [`shrink`].
    ///
    /// [`shrink`]: struct.ArenaSet.html#method.shrink
    pub fn shrink<T: Map<Key = I, Value = I>>(&mut self) -> T
    {
        let ref mut this = self.0;
        shrink!(this, T)
    }
}

/// Errors that may occur when using a [`ArenaSet`].
/// [`ArenaSet`]: struct.ArenaSet.html
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Error {
    /// The ID does not represent an interned item. This could mean either that the ID
    /// has never been returned from a call to [`intern`]; or was subsequently passed
    /// to [`disintern`].
    ///
    /// [`intern`]: struct.ArenaSet.html#method.intern
    /// [`disintern`]: struct.ArenaSet.html#method.disintern
    InvalidId,

    /// Could not convert an ID type to a `Vec` index.
    FromIdFailed,

    /// Could not convert a `Vec` index to an ID type.
    ToIdFailed,

    /// The ID type cannot uniquely represent any more items.
    ///
    /// For instance, if `I = u8`, and there are 256 items in a [`ArenaSet`],
    /// further calls to [`intern`] will fail with this error.
    ///
    /// [`ArenaSet`]: struct.ArenaSet.html
    /// [`intern`]: struct.ArenaSet.html#method.intern
    IdOverflow,
}

// Aside: it'd be really cool if the Rust compiler could figure out that
// `Slot<String>` can be represented by 24 instead of 32 bytes on x86-64.
// Because the heap pointer in `String` is `NonZero`, that can be used as
// a discriminant, and `Vacant` can share space with the adjacent fields.
#[derive(Clone)]
enum Slot<T> {
    Vacant(usize),
    Occupied(T),
}

// trust us, we're engineers
fn make_static<T: ? Sized>(t: &T) -> &'static T {
    unsafe { &*(t as *const T) }
}
