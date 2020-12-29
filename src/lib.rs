//! const fn combinations iter adapter
//!
//! # Examples
//!
//! ```
//! for [n1, n2, n3] in (1..5).combinations() {
//!     println!("{}, {}, {}", n1, n2, n3);
//! }
//! ```

#![feature(min_const_generics)]

use std::iter::Iterator;

trait IterExt: Iterator + Clone + Sized
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
}

impl<I, const N: usize> Iterator for Combinations<I, N>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; N];

    // pub fn k(&self) -> usize { self.indices.len() }
    //  pub fn n(&self) -> usize { self.pool.len() }
    fn next(&mut self) -> Option<[<I as Iterator>::Item; N]> {
        // if self.first {
        //     if self.current_n
        // }
        todo!();
    }
}

use std::ops::Index;

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
