use super::Combinations;
use std::iter::Iterator;

#[derive(Clone, Debug)]
struct FullPermutations<T, const N: usize> {
    a: [T; N],
    c: [usize; N],
    i: usize,
    first: bool,
}

impl<T, const N: usize> FullPermutations<T, N> {
    fn new(a: [T; N]) -> Self {
        Self {
            a,
            c: [0; N],
            i: 0,
            first: true,
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
            Some(self.a.clone())
        } else {
            while self.i < N {
                if self.c[self.i] < self.i {
                    let swap_i = if self.i & 1 == 0 { 0 } else { self.c[self.i] };
                    self.a.swap(swap_i, self.i);
                    self.c[self.i] += 1;
                    self.i = 0;
                    return Some(self.a.clone());
                } else {
                    self.c[self.i] = 0;
                    self.i += 1;
                }
            }
            None
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
        let mut iter = Combinations::new(iter);
        let perm_iter = iter.next().map(FullPermutations::new);
        Self { iter, perm_iter }
    }
}

#[cfg(test)]
mod test {
    use crate::IterExt;

    #[test]
    fn none_on_size_too_big() {
        let mut permutations = (1..2).permutations::<2>();
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut permutations = (1..5).permutations();
        assert_eq!(permutations.next(), Some([]));
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
