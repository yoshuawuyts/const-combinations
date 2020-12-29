//! const fn combinations iter adapter
//!
//! # Examples
//!
//! ```
//! use const_combinations::IterExt;
//!
//! for [n1, n2, n3] in (1..5).combinations() {
//!     println!("{}, {}, {}", n1, n2, n3);
//! }
//! ```

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
    current_n: usize,
    indexes: [usize; N],
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

        Self {
            current_n: 0,
            indexes: [0; N],
            pool,
            first: true,
        }
    }

    pub fn k(&self) -> usize {
        self.current_n
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
        } else if self.indexes.is_empty() {
            return None;
        } else {
            // Scan from the end, looking for an index to increment
            let mut i: usize = self.current_n;

            // Check if we need to consume more from the iterator
            if self.indexes[i] == self.pool.len() - 1 {
                self.pool.get_next(); // may change pool size
            }

            while self.indexes[i] == i + self.pool.len() - self.indexes.len() {
                if i > 0 {
                    i -= 1;
                } else {
                    // Reached the last combination
                    return None;
                }
            }

            // Increment index, and reset the ones to its right
            self.indexes[i] += 1;
            for j in i + 1..self.indexes.len() {
                self.indexes[j] = self.indexes[j - 1] + 1;
            }
        }

        // Create result vector based on the indexes
        let mut out: [MaybeUninit<I::Item>; N] = MaybeUninit::uninit_array();
        self.indexes
            .iter()
            .for_each(|i| out[*i] = MaybeUninit::new(self.pool[*i].clone()));
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
