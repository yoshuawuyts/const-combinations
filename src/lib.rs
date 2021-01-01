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
//! ```

#![feature(maybe_uninit_uninit_array)]

use std::iter::Iterator;
use std::mem::MaybeUninit;

pub trait IterExt: Iterator + Clone + Sized
where
    <Self as Iterator>::Item: Clone,
{
    fn combinations<const N: usize>(self) -> Combinations<Self, N> {
        Combinations::new(self)
    }
}

impl<I> IterExt for I
where
    I: Clone + Iterator,
    <I as Iterator>::Item: Clone,
{
}

pub struct Combinations<I, const N: usize>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    iter: I,
    buffer: Vec<I::Item>,
    indices: [usize; N],
    first: bool,
}

impl<I, const N: usize> Combinations<I, N>
where
    I: Clone + Iterator,
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

        Self {
            indices,
            first: true,
            iter,
            buffer,
        }
    }
}

impl<I, const N: usize> Iterator for Combinations<I, N>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<[<I as Iterator>::Item; N]> {
        if self.first {
            // Validate N never exceeds the total no. of items in the iterator
            if N > self.buffer.len() {
                return None;
            }
            self.first = false;
        } else if N == 0 {
            return None;
        } else {
            // Check if we need to consume more from the iterator
            if self.indices[0] == self.buffer.len() - N {
                // Exhausted all combinations in the current buffer
                match self.iter.next() {
                    Some(x) => self.buffer.push(x),
                    None => return None,
                }
            }

            let mut i = 0;
            // Find the last consecutive index
            while i < N - 1 && self.indices[i] + 1 == self.indices[i + 1] {
                i += 1;
            }
            self.indices[i] += 1;
            // Increment index, and reset the ones to its left
            for j in 0..i {
                self.indices[j] = j;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combinations_order() {
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
    }

    #[test]
    fn none_on_size_too_big() {
        let mut combinations = (1..2).combinations::<2>();
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut combinations = (1..5).combinations();
        assert_eq!(combinations.next(), Some([]));
        assert_eq!(combinations.next(), None);
    }
}
