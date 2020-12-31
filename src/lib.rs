//! const fn combinations iter adapter
//!
//! # Examples
//!
//! ```
//! use const_combinations::IterExt;
//!
//! let mut combinations = (1..5).combinations();
//! assert_eq!(combinations.next(), Some([1, 2, 3]));
//! assert_eq!(combinations.next(), Some([1, 2, 4]));
//! assert_eq!(combinations.next(), Some([1, 3, 4]));
//! assert_eq!(combinations.next(), Some([2, 3, 4]));
//! assert_eq!(combinations.next(), None);
//!
//! let mut permutations = (0..3).permutations();
//! assert_eq!(permutations.next(), Some([0, 1]));
//! assert_eq!(permutations.next(), Some([1, 0]));
//! assert_eq!(permutations.next(), Some([0, 2]));
//! assert_eq!(permutations.next(), Some([2, 0]));
//! assert_eq!(permutations.next(), Some([1, 2]));
//! assert_eq!(permutations.next(), Some([2, 1]));
//! assert_eq!(permutations.next(), None);
//!
//! ```

#![feature(maybe_uninit_uninit_array)]

use std::iter::Iterator;
use std::mem::MaybeUninit;

pub trait IterExt: Iterator + Sized
where
    Self::Item: Clone,
{
    fn combinations<const N: usize>(self) -> Combinations<Self, N> {
        Combinations::new(self)
    }

    fn permutations<const N: usize>(self) -> Permutations<Self, N> {
        Permutations::new(self)
    }
}

impl<I> IterExt for I
where
    I: Iterator,
    I::Item: Clone,
{
}

pub struct Combinations<I, const N: usize>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: I,
    done: bool,
    buffer: Vec<I::Item>,
    indices: [usize; N],
    first: bool,
}

impl<I, const N: usize> Combinations<I, N>
where
    I: Iterator,
    I::Item: Clone,
{
    fn new(mut iter: I) -> Self {
        // Prepare the indices.
        let mut indices = [0; N];
        for i in 0..N {
            indices[i] = i;
        }

        // Prefill the buffer.
        let buffer: Vec<I::Item> = iter.by_ref().take(N).collect();
        let done = buffer.len() < N;

        Self {
            indices,
            first: true,
            iter,
            done,
            buffer,
        }
    }
}

impl<I, const N: usize> Iterator for Combinations<I, N>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<[I::Item; N]> {
        if self.first {
            // Validate N never exceeds the total no. of items in the iterator
            if N > self.buffer.len() {
                return None;
            }
            self.first = false;
        } else if N == 0 {
            return None;
        } else {
            // Scan from the end, looking for an index to increment
            let mut i: usize = N - 1;

            // Check if we need to consume more from the iterator
            if !self.done && self.indices[i] == self.buffer.len() - 1 {
                match self.iter.next() {
                    Some(x) => self.buffer.push(x),
                    None => self.done = true,
                }
            }

            while self.indices[i] == i + self.buffer.len() - N {
                if i > 0 {
                    i -= 1;
                } else {
                    // Reached the last combination
                    return None;
                }
            }

            // Increment index, and reset the ones to its right
            self.indices[i] += 1;
            for j in i + 1..N {
                self.indices[j] = self.indices[j - 1] + 1;
            }
        }

        // Create result vector based on the indexes
        let mut out: [MaybeUninit<I::Item>; N] = MaybeUninit::uninit_array();
        self.indices.iter().enumerate().for_each(|(oi, i)| {
            out[oi] = MaybeUninit::new(self.buffer[*i].clone());
        });
        Some(unsafe { out.as_ptr().cast::<[I::Item; N]>().read() })
    }
}

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
            match perm_iter.next() {
                Some(a) => Some(a),
                None => {
                    self.perm_iter = self.iter.next().map(FullPermutations::new);
                    // Each `FullPermutations` is guaranteed to return at least one item
                    self.perm_iter.as_mut().and_then(|i| i.next())
                }
            }
        } else {
            None
        }
    }
}

impl<I, const K: usize> Permutations<I, K>
where
    I: Iterator,
    I::Item: Clone,
{
    fn new(iter: I) -> Self {
        let mut iter = Combinations::new(iter);
        let perm_iter = iter.next().map(FullPermutations::new);
        Self { iter, perm_iter }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combinations_none_on_size_too_big() {
        let mut combinations = (1..2).combinations::<2>();
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn combinations_empty_arr_on_n_zero() {
        let mut combinations = (1..5).combinations();
        assert_eq!(combinations.next(), Some([]));
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn permutations_none_on_size_too_big() {
        let mut permutations = (1..2).permutations::<2>();
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn permutations_empty_arr_on_n_zero() {
        let mut permutations = (1..5).permutations();
        assert_eq!(permutations.next(), Some([]));
        assert_eq!(permutations.next(), None);
    }
}
