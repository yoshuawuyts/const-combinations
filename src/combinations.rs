use alloc::vec::Vec;
use core::iter::{FusedIterator, Iterator};

#[derive(Clone)]
pub struct LazyCombinationGenerator<const K: usize> {
    indices: [usize; K],
    done: bool,
}

impl<const K: usize> LazyCombinationGenerator<K> {
    pub fn new() -> Self {
        Self {
            indices: crate::make_array(|i| i),
            done: false,
        }
    }

    pub fn max_index(&self) -> Option<usize> {
        self.indices.last().copied()
    }

    pub fn is_done(&self, item_count: usize) -> bool {
        self.done || self.max_index() >= Some(item_count)
    }

    pub fn indices(&self) -> &[usize; K] {
        &self.indices
    }

    pub fn step(&mut self) {
        if K == 0 {
            self.done = true;
        } else {
            let mut i = 0;
            // Reset consecutive indices
            while i + 1 < K && self.indices[i] + 1 == self.indices[i + 1] {
                self.indices[i] = i;
                i += 1;
            }
            // Increment the last consecutive index
            self.indices[i] += 1;
        }
    }
}

/// An iterator that returns k-length combinations of values from `iter`.
///
/// This `struct` is created by the [`combinations`] method on [`IterExt`]. See its
/// documentation for more.
///
/// [`combinations`]: super::IterExt::combinations
/// [`IterExt`]: super::IterExt
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Combinations<I, const K: usize>
where
    I: Iterator,
{
    iter: I,
    items: Vec<I::Item>,
    gen: LazyCombinationGenerator<K>,
}

impl<I, const K: usize> Combinations<I, K>
where
    I: Iterator,
{
    pub(crate) fn new(iter: I) -> Self {
        Self {
            items: Vec::new(),
            iter,
            gen: LazyCombinationGenerator::new(),
        }
    }
}

impl<I, const K: usize> Iterator for Combinations<I, K>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; K];

    fn next(&mut self) -> Option<[I::Item; K]> {
        // Check if we need to consume more from the iterator
        let missing_count = self
            .gen
            .max_index()
            .map(|m| (m + 1).saturating_sub(self.items.len()))
            .unwrap_or_default();
        if missing_count > 0 {
            // Try to fill the buffer
            self.items.extend(self.iter.by_ref().take(missing_count));
        }

        if self.gen.is_done(self.items.len()) {
            None
        } else {
            let indices = self.gen.indices();
            let res = make_array(|i| self.items[indices[i]].clone());
            self.gen.step();
            Some(res)
        }
    }
}

impl<I, const K: usize> FusedIterator for Combinations<I, K>
where
    I: FusedIterator,
    I::Item: Clone,
{
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IterExt;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn order() {
        let mut combinations = (1..6).combinations();
        assert_eq!(combinations.next(), Some([1, 2, 3]));
        assert_eq!(combinations.next(), Some([1, 2, 4]));
        assert_eq!(combinations.next(), Some([1, 3, 4]));
        assert_eq!(combinations.next(), Some([2, 3, 4]));
        assert_eq!(combinations.next(), Some([1, 2, 5]));
        assert_eq!(combinations.next(), Some([1, 3, 5]));
        assert_eq!(combinations.next(), Some([2, 3, 5]));
        assert_eq!(combinations.next(), Some([1, 4, 5]));
        assert_eq!(combinations.next(), Some([2, 4, 5]));
        assert_eq!(combinations.next(), Some([3, 4, 5]));
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn none_on_size_too_big() {
        let mut combinations = (1..2).combinations::<2>();
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut combinations = (1..5).combinations();
        assert_eq!(combinations.next(), Some([]));
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn fused_propagation() {
        let fused = [1, 2, 3].iter().fuse();
        let combinations = fused.combinations::<2>();

        fn is_fused<T: FusedIterator>(_: T) {}
        is_fused(combinations);
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
        let mut combinations = ResumeIter {
            items: &[1, 2, 3, 4],
            len: &len,
            i: 0,
        }
        .combinations();

        assert_eq!(combinations.next(), None);

        len.fetch_add(1, Ordering::SeqCst);
        assert_eq!(combinations.next(), None);

        len.fetch_add(1, Ordering::SeqCst);
        assert_eq!(combinations.next(), None);

        len.fetch_add(1, Ordering::SeqCst);
        assert_eq!(combinations.next(), Some([1, 2, 3]));
        assert_eq!(combinations.next(), None);

        len.fetch_add(1, Ordering::SeqCst);
        assert_eq!(combinations.next(), Some([1, 2, 4]));
        assert_eq!(combinations.next(), Some([1, 3, 4]));
        assert_eq!(combinations.next(), Some([2, 3, 4]));
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }
}
