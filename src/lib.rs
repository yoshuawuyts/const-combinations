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

#![allow(stable_features)]
#![feature(min_const_generics)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]

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
    pool: LazyBuffer<I>,
    indices: [usize; N],
    first: bool,
}

impl<I, const N: usize> Combinations<I, N>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    fn new(iter: I) -> Self {
        let mut pool = LazyBuffer::new(iter);
        pool.prefill(N);

        let mut indices = [0; N];
        for i in 0..N {
            indices[i] = i;
        }

        Self {
            indices,
            pool,
            first: true,
        }
    }

    pub fn k(&self) -> usize {
        N
    }
    pub fn n(&self) -> usize {
        self.pool.len()
    }
}

impl<I, const N: usize> Iterator for Combinations<I, N>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; N];

    // This impl was copied from:
    // https://docs.rs/itertools/0.10.0/src/itertools/combinations.rs
    fn next(&mut self) -> Option<[<I as Iterator>::Item; N]> {
        if self.first {
            if self.k() > self.n() {
                return None;
            }
            self.first = false;
        } else if N == 0 {
            return None;
        } else {
            // Scan from the end, looking for an index to increment
            let mut i: usize = N - 1;

            // Check if we need to consume more from the iterator
            if self.indices[i] == self.pool.len() - 1 {
                self.pool.get_next(); // may change pool size
            }

            while self.indices[i] == i + self.pool.len() - N {
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
        self.indices
            .iter()
            .enumerate()
            .for_each(|(oi, i)| out[oi] = MaybeUninit::new(self.pool[*i].clone()));
        Some(unsafe { out.as_ptr().cast::<[I::Item; N]>().read() })
    }
}

use std::ops::Index;

// This impl was copied from:
// https://docs.rs/itertools/0.10.0/src/itertools/lazy_buffer.rs.html
#[derive(Debug, Clone)]
pub struct LazyBuffer<I: Iterator> {
    pub it: I,
    done: bool,
    buffer: Vec<I::Item>,
}

impl<I> LazyBuffer<I>
where
    I: Iterator,
{
    pub fn new(it: I) -> LazyBuffer<I> {
        LazyBuffer {
            it,
            done: false,
            buffer: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_next(&mut self) -> bool {
        if self.done {
            return false;
        }
        let next_item = self.it.next();
        match next_item {
            Some(x) => {
                self.buffer.push(x);
                true
            }
            None => {
                self.done = true;
                false
            }
        }
    }

    pub fn prefill(&mut self, len: usize) {
        let buffer_len = self.buffer.len();

        if !self.done && len > buffer_len {
            let delta = len - buffer_len;

            self.buffer.extend(self.it.by_ref().take(delta));
            self.done = self.buffer.len() < len;
        }
    }
}

impl<I, J> Index<J> for LazyBuffer<I>
where
    I: Iterator,
    I::Item: Sized,
    Vec<I::Item>: Index<J>,
{
    type Output = <Vec<I::Item> as Index<J>>::Output;

    fn index(&self, _index: J) -> &Self::Output {
        self.buffer.index(_index)
    }
}
