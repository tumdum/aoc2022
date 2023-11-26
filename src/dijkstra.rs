use rustc_hash::FxHashMap;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::hash::Hash;

// Returns cost of path from start and previous nodes for path reconstruction.
pub fn dijkstra<T>(
    start: T,
    neighbours_of: impl Fn(&T) -> Vec<(T, u64)>,
) -> (FxHashMap<T, u64>, FxHashMap<T, T>)
where
    T: Debug + PartialEq + Eq + PartialOrd + Ord + Hash + Clone,
{
    let mut dist: FxHashMap<T, u64> = Default::default();
    dist.insert(start.clone(), 0);

    let mut prev: FxHashMap<T, T> = Default::default();

    #[derive(Debug, PartialEq, Eq, Ord)]
    struct State<U: Debug + PartialEq + Eq + PartialOrd + Ord> {
        key: U,
        prio: u64,
    }

    impl<U: Debug + PartialEq + Eq + PartialOrd + Ord> PartialOrd for State<U> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            let o = self.prio.partial_cmp(&other.prio);
            if o == Some(Ordering::Equal) {
                self.key.partial_cmp(&other.key)
            } else {
                o
            }
        }
    }
    let mut todo: BinaryHeap<Reverse<State<T>>> = BinaryHeap::default();
    todo.push(Reverse(State {
        key: start.clone(),
        prio: *dist.get(&start).unwrap(),
    }));

    while let Some(Reverse(State { key, prio })) = todo.pop() {
        for (neighbour, cost) in neighbours_of(&key) {
            debug_assert_eq!(&prio, dist.get(&key).unwrap());
            let alt = prio + cost;
            if alt < *dist.get(&neighbour).unwrap_or(&u64::MAX) {
                dist.insert(neighbour.clone(), alt);
                prev.insert(neighbour.clone(), key.clone());
                todo.push(Reverse(State {
                    key: neighbour,
                    prio: alt,
                }));
            }
        }
    }
    (dist, prev)
}
