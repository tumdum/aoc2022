use rustc_hash::FxHashMap;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub fn path<T: PartialEq + Eq + Hash + Clone>(
    from: &T,
    to: &T,
    prev: &FxHashMap<T, T>,
) -> Option<Vec<T>> {
    let mut path: Vec<T> = vec![to.clone()];
    while path.last().unwrap() != from {
        if let Some(prev) = prev.get(path.last().unwrap()) {
            path.push(prev.clone());
        } else {
            return None;
        }
    }
    path.reverse();
    Some(path)
}

// Returns cost of path from start and previous nodes for path reconstruction.
pub fn bfs<T>(
    start: T,
    is_target: impl Fn(&T) -> bool,
    neighbours_of: impl Fn(&T) -> Vec<T>,
) -> FxHashMap<T, T>
where
    T: Debug + PartialEq + Eq + PartialOrd + Ord + Hash + Clone,
{
    let mut prev: FxHashMap<T, T> = Default::default();
    let mut todo: VecDeque<T> = Default::default();
    todo.push_back(start.clone());

    while let Some(next) = todo.pop_front() {
        if is_target(&next) {
            assert!(next == start || prev.get(&next).is_some());
            return prev;
        }
        for candidate in neighbours_of(&next) {
            if prev.get(&candidate).is_some() {
                continue;
            }
            todo.push_back(candidate.clone());
            prev.insert(candidate, next.clone());
        }
    }

    prev
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest_derive::Arbitrary;
    use std::collections::{HashMap, HashSet};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Arbitrary, Hash, PartialOrd, Ord)]
    enum Node {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
    }
    use Node::*;
    const ALL: [Node; 12] = [A, B, C, D, E, F, G, H, I, J, K, L];

    #[test]
    fn full_graph() {
        let start = A;
        let n_of = |_: &Node| ALL.into_iter().map(|n| (n, 1)).collect();
        let (_, d_prev) = dijkstra(start, n_of);
        for target in ALL {
            let b_prev = bfs(start, |n| n == &target, |_| ALL.into_iter().collect());
            let d_path = path(&start, &target, &d_prev);
            let b_path = path(&start, &target, &b_prev);
            match (d_path, b_path) {
                (Some(dp), Some(bp)) => {
                    assert_eq!(dp.len(), bp.len());
                }
                (None, None) => {}
                _ => panic!(),
            }
        }
    }

    #[test]
    fn chain() {
        let start = *ALL.first().unwrap();
        let target = *ALL.last().unwrap();
        let mut nodes: HashMap<Node, Vec<Node>> = Default::default();
        for v in ALL.as_slice().windows(2) {
            nodes.insert(v[0], vec![v[1]]);
        }
        let n_of = |n: &Node| {
            nodes
                .get(n)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|n| (n, 1))
                .collect::<Vec<_>>()
        };
        let (_, d_prev) = dijkstra(start, n_of);
        let b_prev = bfs(
            start,
            |n| n == &target,
            |n| {
                nodes
                    .get(n)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<Vec<_>>()
            },
        );
        let d_path = path(&start, &target, &d_prev);
        let b_path = path(&start, &target, &b_prev);
        assert_eq!(d_path, b_path);
    }

    proptest! {
        #[test]
        fn bfs_and_dijkstra_equal(nodes: HashMap<Node, HashSet<Node>>, start: Node) {
            let n_of = |n: &Node| nodes.get(n).cloned().unwrap_or_default().into_iter().map(|n| (n, 1)).collect::<Vec<_>>();
            let (_, d_prev) = dijkstra(start, n_of);
            for target in ALL {
                let b_prev = bfs(start, |n| n == &target, |n| nodes.get(n).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>());
                let d_path = path(&start, &target, &d_prev);
                let b_path = path(&start, &target, &b_prev);
                match (d_path, b_path) {
                    (Some(dp), Some(bp)) => {
                        assert_eq!(dp.len(), bp.len());
                    }
                    (None, None) => {},
                    _ => panic!(),
                }
            }

        }
    }
}
