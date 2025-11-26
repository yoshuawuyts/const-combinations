use crate::combinations::LazyCombinationGenerator;
use alloc::vec::Vec;
use core::iter::{FusedIterator, Iterator};

#[derive(Clone)]
pub struct LazyPermutationGenerator<const N: usize> {
    indices: [usize; N],
    counters: [usize; N],
    done: bool,
}

impl<const N: usize> LazyPermutationGenerator<N> {
    pub fn new() -> Self {
        Self {
            indices: core::array::from_fn(|i| i),
            counters: [0; N],
            done: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn indices(&self) -> &[usize; N] {
        &self.indices
    }

    pub fn step(&mut self) {
        // Iterative version of Heap's algorithm
        // https://en.wikipedia.org/wiki/Heap%27s_algorithm
        let mut i = 1;
        while i < N && self.counters[i] >= i {
            self.counters[i] = 0;
            i += 1;
        }
        if i < N {
            if i & 1 == 0 {
                self.indices.swap(i, 0);
            } else {
                self.indices.swap(i, self.counters[i]);
            };
            self.counters[i] += 1;
        } else {
            self.done = true;
        }
    }
}

#[derive(Clone)]
struct State<const K: usize> {
    comb_gen: LazyCombinationGenerator<K>,
    perm_gen: LazyPermutationGenerator<K>,
}

impl<const K: usize> State<K> {
    fn new() -> Self {
        Self {
            comb_gen: LazyCombinationGenerator::new(),
            perm_gen: LazyPermutationGenerator::new(),
        }
    }

    fn max_index(&self) -> Option<usize> {
        self.comb_gen.max_index()
    }

    fn get_and_step<'a, T, O, F>(&mut self, items: &'a [T], f: F) -> Option<[O; K]>
    where
        F: Fn(&'a T) -> O,
        O: 'a,
    {
        if self.comb_gen.is_done(items.len()) {
            None
        } else {
            let comb_indices = self.comb_gen.indices();
            let perm_indices = self.perm_gen.indices();
            let res = core::array::from_fn(|i| f(&items[comb_indices[perm_indices[i]]]));
            self.perm_gen.step();
            if self.perm_gen.is_done() {
                // Reset the permutation generator and move to the next combination
                self.perm_gen = LazyPermutationGenerator::new();
                self.comb_gen.step();
            }
            Some(res)
        }
    }
}

/// An iterator that returns k-length permutations of values from `iter`.
///
/// This `struct` is created by the [`permutations`] method on [`IterExt`]. See its
/// documentation for more.
///
/// [`permutations`]: super::IterExt::permutations
/// [`IterExt`]: super::IterExt
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Permutations<I, const K: usize>
where
    I: Iterator,
{
    iter: I,
    items: Vec<I::Item>,
    state: State<K>,
}

impl<I, const K: usize> Permutations<I, K>
where
    I: Iterator,
{
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            items: Vec::new(),
            state: State::new(),
        }
    }
}

impl<I, const K: usize> Iterator for Permutations<I, K>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; K];

    fn next(&mut self) -> Option<[I::Item; K]> {
        if K > 0 {
            let max_index = self.state.max_index().unwrap();
            let missing_count = (max_index + 1).saturating_sub(self.items.len());
            if missing_count > 0 {
                // Try to fill the buffer
                self.items.extend(self.iter.by_ref().take(missing_count));
            }
        }
        self.state.get_and_step(&self.items, |t| t.clone())
    }
}

impl<I, const K: usize> FusedIterator for Permutations<I, K>
where
    // This should be `I: Iterator, Combinations<I, K>: FusedIterator`,
    // but it exposes the implementation and makes for lousy docs.
    // There is a test which will stop compiling if the bounds for
    // `FusedIterator for Combinations` impl change.
    I: FusedIterator,
    I::Item: Clone,
{
}

/// An iterator that returns k-length permutations of values from `slice`.
#[derive(Clone)]
#[must_use = "iterators do nothing unless consumed"]
pub struct SlicePermutations<'a, T, const K: usize> {
    items: &'a [T],
    state: State<K>,
}

impl<'a, T, const K: usize> Iterator for SlicePermutations<'a, T, K> {
    type Item = [&'a T; K];

    fn next(&mut self) -> Option<Self::Item> {
        self.state.get_and_step(self.items, |t| t)
    }
}

impl<'a, T, const K: usize> SlicePermutations<'a, T, K> {
    pub(crate) fn new(items: &'a [T]) -> Self {
        Self {
            items,
            state: State::new(),
        }
    }
}

impl<T, const K: usize> FusedIterator for SlicePermutations<'_, T, K> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IterExt;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn order() {
        let mut permutations = (1..4).permutations();
        assert_eq!(permutations.next(), Some([1, 2]));
        assert_eq!(permutations.next(), Some([2, 1]));
        assert_eq!(permutations.next(), Some([1, 3]));
        assert_eq!(permutations.next(), Some([3, 1]));
        assert_eq!(permutations.next(), Some([2, 3]));
        assert_eq!(permutations.next(), Some([3, 2]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn gen_order() {
        let mut gen = LazyPermutationGenerator::new();
        assert_eq!(gen.indices(), &[0, 1, 2]);
        assert!(!gen.is_done());
        gen.step();
        assert_eq!(gen.indices(), &[1, 0, 2]);
        assert!(!gen.is_done());
        gen.step();
        assert_eq!(gen.indices(), &[2, 0, 1]);
        assert!(!gen.is_done());
        gen.step();
        assert_eq!(gen.indices(), &[0, 2, 1]);
        assert!(!gen.is_done());
        gen.step();
        assert_eq!(gen.indices(), &[1, 2, 0]);
        assert!(!gen.is_done());
        gen.step();
        assert_eq!(gen.indices(), &[2, 1, 0]);
        assert!(!gen.is_done());
        gen.step();
        assert!(gen.is_done());
        gen.step();
        assert!(gen.is_done());
    }

    #[test]
    fn none_on_size_too_big() {
        let mut permutations = (1..2).permutations::<2>();
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut permutations = (1..5).permutations();
        assert_eq!(permutations.next(), Some([]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn fused_propagation() {
        let fused = [1, 2, 3].iter().fuse();
        let permutations = fused.permutations::<2>();

        fn is_fused<T: FusedIterator>(_: T) {}
        is_fused(permutations);
    }

    #[test]
    fn resume_after_none() {
        struct ResumeIter<'l, 'a, T>
        where
            T: Copy,
        {
            items: &'a [T],
            i: usize,
            len: &'l AtomicUsize,
        }

        impl<T> Iterator for ResumeIter<'_, '_, T>
        where
            T: Copy,
        {
            type Item = T;
            fn next(&mut self) -> Option<T> {
                if self.i >= self.len.load(Ordering::SeqCst) {
                    None
                } else {
                    self.i += 1;
                    Some(self.items[self.i - 1])
                }
            }
        }

        let len = AtomicUsize::new(0);
        let mut permutations = ResumeIter {
            items: &[1, 2, 3],
            len: &len,
            i: 0,
        }
        .permutations();

        assert_eq!(permutations.next(), None);

        len.fetch_add(1, Ordering::SeqCst);
        assert_eq!(permutations.next(), None);

        len.fetch_add(1, Ordering::SeqCst);
        assert_eq!(permutations.next(), Some([1, 2]));
        assert_eq!(permutations.next(), Some([2, 1]));
        assert_eq!(permutations.next(), None);

        len.fetch_add(1, Ordering::SeqCst);
        assert_eq!(permutations.next(), Some([1, 3]));
        assert_eq!(permutations.next(), Some([3, 1]));
        assert_eq!(permutations.next(), Some([2, 3]));
        assert_eq!(permutations.next(), Some([3, 2]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }
}

#[cfg(test)]
mod slice_test {
    use crate::SliceExt;

    #[test]
    fn order() {
        let mut permutations = [1, 2, 3].permutations();
        assert_eq!(permutations.next(), Some([&1, &2]));
        assert_eq!(permutations.next(), Some([&2, &1]));
        assert_eq!(permutations.next(), Some([&1, &3]));
        assert_eq!(permutations.next(), Some([&3, &1]));
        assert_eq!(permutations.next(), Some([&2, &3]));
        assert_eq!(permutations.next(), Some([&3, &2]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn none_on_size_too_big() {
        let mut permutations = [1].permutations::<2>();
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut permutations = [1, 2, 3, 4].permutations();
        assert_eq!(permutations.next(), Some([]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }
}
