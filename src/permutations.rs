use super::Combinations;
use std::iter::{FusedIterator, Iterator};

#[derive(Clone, Debug)]
struct FullPermutations<T, const N: usize> {
    items: [T; N],
    indices: [usize; N],
    first: bool,
    done: bool,
}

impl<T, const N: usize> FullPermutations<T, N> {
    fn new(items: [T; N]) -> Self {
        Self {
            items,
            indices: [0; N],
            first: true,
            done: false,
        }
    }
}

impl<T, const N: usize> Iterator for FullPermutations<T, N>
where
    T: Clone,
{
    type Item = [T; N];

    fn next(&mut self) -> Option<Self::Item> {
        // Iterative version of Heap's algorithm
        // https://en.wikipedia.org/wiki/Heap%27s_algorithm
        if self.first {
            self.first = false;
        } else if self.done {
            return None;
        } else {
            let mut i = 1;
            while i < N && self.indices[i] >= i {
                self.indices[i] = 0;
                i += 1;
            }
            if i >= N {
                self.done = true;
                return None;
            }
            if i & 1 == 0 {
                self.items.swap(i, 0);
            } else {
                self.items.swap(i, self.indices[i]);
            };
            self.indices[i] += 1;
        }
        Some(self.items.clone())
    }
}

/// An iterator that returns k-length permutations of values from `iter`.
///
/// This `struct` is created by the [`permutations`] method on [`IterExt`]. See its
/// documentation for more.
///
/// [`permutations`]: super::IterExt::permutations
/// [`IterExt`]: super::IterExt
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Permutations<I, const K: usize>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: Combinations<I, K>,
    perm_iter: Option<FullPermutations<I::Item, K>>,
}

impl<I, const K: usize> Iterator for Permutations<I, K>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; K];

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(perm_iter) = &mut self.perm_iter {
            if let Some(a) = perm_iter.next() {
                return Some(a);
            }
        }
        self.perm_iter = self.iter.next().map(FullPermutations::new);
        // Each `FullPermutations` is guaranteed to return at least one item.
        // `None` will be returned only if `perm_iter` is `None`,
        // which means that the inner iterator returned `None`.
        self.perm_iter.as_mut().and_then(|i| i.next())
    }
}

impl<I, const K: usize> Permutations<I, K>
where
    I: Iterator,
    I::Item: Clone,
{
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter: Combinations::new(iter),
            perm_iter: None,
        }
    }
}

impl<I, const K: usize> FusedIterator for Permutations<I, K>
where
    // This should be `I: Iterator, Combinations<I, K>: FusedIterator`,
    // but it exposes the implementation and makes for lousy docs.
    // I added a comment to `FusedIterator for Combinations`
    // to change it here as well if it changes there.
    I: FusedIterator,
    I::Item: Clone,
{
}

#[cfg(test)]
mod test {
    use crate::IterExt;

    use super::FullPermutations;

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
    fn full_order() {
        let mut permutations = FullPermutations::new([1, 2, 3]);
        assert_eq!(permutations.next(), Some([1, 2, 3]));
        assert_eq!(permutations.next(), Some([2, 1, 3]));
        assert_eq!(permutations.next(), Some([3, 1, 2]));
        assert_eq!(permutations.next(), Some([1, 3, 2]));
        assert_eq!(permutations.next(), Some([2, 3, 1]));
        assert_eq!(permutations.next(), Some([3, 2, 1]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
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
    fn resume_after_none() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut permutations = receiver.try_iter().permutations();
        assert_eq!(permutations.next(), None);

        sender.send(1).unwrap();
        assert_eq!(permutations.next(), None);

        sender.send(2).unwrap();
        assert_eq!(permutations.next(), Some([1, 2]));
        assert_eq!(permutations.next(), Some([2, 1]));
        assert_eq!(permutations.next(), None);

        sender.send(3).unwrap();
        assert_eq!(permutations.next(), Some([1, 3]));
        assert_eq!(permutations.next(), Some([3, 1]));
        assert_eq!(permutations.next(), Some([2, 3]));
        assert_eq!(permutations.next(), Some([3, 2]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }
}
