//! const fn combinations iter adapter

#![feature(maybe_uninit_uninit_array)]

mod combinations;
mod permutations;

pub use combinations::Combinations;
pub use permutations::Permutations;

/// An extension trait adding `combinations` and `permutations` to `Iterator`.
pub trait IterExt: Iterator {
    /// Return an iterator adaptor that iterates over the k-length combinations of
    /// the elements from an iterator.
    ///
    /// The iterator produces a new array per iteration, and clones the iterator
    /// elements. If `K` is greater than the length of the input iterator the
    /// resulting iterator adaptor will yield no items.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_combinations::IterExt;
    ///
    /// let mut combinations = (1..5).combinations();
    /// assert_eq!(combinations.next(), Some([1, 2, 3]));
    /// assert_eq!(combinations.next(), Some([1, 2, 4]));
    /// assert_eq!(combinations.next(), Some([1, 3, 4]));
    /// assert_eq!(combinations.next(), Some([2, 3, 4]));
    /// assert_eq!(combinations.next(), None);
    /// ```
    ///
    /// Note: Combinations does not take into account the equality of the iterated values.
    ///
    /// ```
    /// # use const_combinations::IterExt;
    /// let mut combinations = vec![1, 2, 2].into_iter().combinations();
    /// assert_eq!(combinations.next(), Some([1, 2])); // Note: these are the same
    /// assert_eq!(combinations.next(), Some([1, 2])); // Note: these are the same
    /// assert_eq!(combinations.next(), Some([2, 2]));
    /// assert_eq!(combinations.next(), None);
    /// ```
    fn combinations<const K: usize>(self) -> Combinations<Self, K>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        Combinations::new(self)
    }

    /// Return an iterator adaptor that iterates over the k-length permutations of
    /// the elements from an iterator.
    ///
    /// The iterator produces a new array per iteration, and clones the iterator
    /// elements. If `K` is greater than the length of the input iterator the
    /// resulting iterator adaptor will yield no items.
    ///
    /// # Examples
    ///
    /// ```
    /// # use const_combinations::IterExt;
    /// let mut permutations = (0..3).permutations();
    /// assert_eq!(permutations.next(), Some([0, 1]));
    /// assert_eq!(permutations.next(), Some([1, 0]));
    /// assert_eq!(permutations.next(), Some([0, 2]));
    /// assert_eq!(permutations.next(), Some([2, 0]));
    /// assert_eq!(permutations.next(), Some([1, 2]));
    /// assert_eq!(permutations.next(), Some([2, 1]));
    /// assert_eq!(permutations.next(), None);
    /// ```
    ///
    /// Note: Permutations does not take into account the equality of the iterated values.
    ///
    /// ```
    /// # use const_combinations::IterExt;
    /// let mut permutations = vec![2, 2].into_iter().permutations();
    /// assert_eq!(permutations.next(), Some([2, 2])); // Note: these are the same
    /// assert_eq!(permutations.next(), Some([2, 2])); // Note: these are the same
    /// assert_eq!(permutations.next(), None);
    /// ```
    fn permutations<const K: usize>(self) -> Permutations<Self, K>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        Permutations::new(self)
    }
}

impl<I> IterExt for I where I: Iterator {}
