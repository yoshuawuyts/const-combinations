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
    done: bool,
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
        let mut buffer = Vec::with_capacity(N);
        let mut done = false;
        if N > buffer.len() {
            buffer.extend(iter.by_ref().take(N - buffer.len()));
            done = buffer.len() < N;
        }

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

#[cfg(test)]
mod test {
    use super::*;

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
