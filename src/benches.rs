use std::collections::{BTreeMap, HashMap};
use rand::{thread_rng, Rng};
use super::*;
use test::Bencher;
use traits::Map;

fn prefilled<M: Map<Key = &'static str, Value = u32>, F, P>(count: u32, f: F) -> (ArenaSet<String, u32, M>, Vec<P>)
    where F: Fn(String, u32) -> P {
    let mut rng = thread_rng();
    let mut p = ArenaSet::new().unwrap();
    let mut path = Vec::new();
    for i in 0..count {
        let s = format!("{}{}", rng.next_u64(), i);
        p.intern(s.as_str()).unwrap();
        path.push(f(s, i));
    }
    p.intern("hello").unwrap();
    path.push(f(String::from("hello"), count));
    thread_rng().shuffle(&mut path);
    (p, path)
}

// spoiler: all combinations are just as fast!
/*
fn bench_resolve_existing<M: Map<Key = &'static str, Value = u32>>(b: &mut Bencher, count: u32) {
    let (p, path) = prefilled::<M, _, _>(count, |_, i| { i });
    let mut it = path.iter().cycle();
    b.iter(|| {
        let _: &str = p.resolve(it.next().unwrap()).unwrap();
    });
}
*/

fn bench_intern_existing<M: Map<Key = &'static str, Value = u32>>(b: &mut Bencher, count: u32) {
    let (mut p, path) = prefilled::<M, _, _>(count, |s, _| { s });
    let mut it = path.iter().cycle();
    b.iter(|| {
        p.intern(it.next().unwrap().as_str()).unwrap();
    });
}

fn bench_intern_disintern<M: Map<Key = &'static str, Value = u32>>(b: &mut Bencher, count: u32) {
    let (mut p, path) = prefilled::<M, _, _>(count, |s, i| { (s, i) });
    let (mut strings, mut ids): (Vec<_>, Vec<_>) = path.into_iter().unzip();
    thread_rng().shuffle(&mut strings);
    thread_rng().shuffle(&mut ids);
    let mut string = strings.iter().cycle();
    let mut id = ids.into_iter().cycle();
    b.iter(|| {
        let s = string.next().unwrap();
        p.intern(s.as_str()).unwrap();
        p.disintern(id.next().unwrap()).ok();
    });
}

macro_rules! benchset {
        ( $t:ty, $c:expr, $n1:ident $n2:ident) => {
            #[bench]
            fn $n1(b: &mut Bencher) {
                bench_intern_existing::<$t>(b, $c);
            }

            #[bench]
            fn $n2(b: &mut Bencher) {
                bench_intern_disintern::<$t>(b, $c);
            }
        };
    }

benchset!(HashMap<&'static str, u32>, 0,
        bench_intern_existing_hash_map_empty
        bench_intern_disintern_hash_map_empty);

benchset!(HashMap<&'static str, u32>, 10,
        bench_intern_existing_hash_map_prefilled10
        bench_intern_disintern_hash_map_prefilled10);

benchset!(HashMap<&'static str, u32>, 100,
        bench_intern_existing_hash_map_prefilled100
        bench_intern_disintern_hash_map_prefilled100);

benchset!(HashMap<&'static str, u32>, 1000,
        bench_intern_existing_hash_map_prefilled1000
        bench_intern_disintern_hash_map_prefilled1000);

benchset!(HashMap<&'static str, u32>, 10000,
        bench_intern_existing_hash_map_prefilled10000
        bench_intern_disintern_hash_map_prefilled10000);

benchset!(BTreeMap<&'static str, u32>, 0,
        bench_intern_existing_btree_map_empty
        bench_intern_disintern_btree_map_empty);

benchset!(BTreeMap<&'static str, u32>, 10,
        bench_intern_existing_btree_map_prefilled10
        bench_intern_disintern_btree_map_prefilled10);

benchset!(BTreeMap<&'static str, u32>, 100,
        bench_intern_existing_btree_map_prefilled100
        bench_intern_disintern_btree_map_prefilled100);

benchset!(BTreeMap<&'static str, u32>, 1000,
        bench_intern_existing_btree_map_prefilled1000
        bench_intern_disintern_btree_map_prefilled1000);

benchset!(BTreeMap<&'static str, u32>, 10000,
        bench_intern_existing_btree_map_prefilled10000
        bench_intern_disintern_btree_map_prefilled10000);
