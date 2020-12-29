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

trait IterExt: std::iter::Iterator + Sized {
    fn combinations<const N: usize>(&self) -> Combinations<'_, Self::Item, N> {
        Combinations::new(self)
    }
}

impl<T> IterExt for T where T: std::iter::Iterator {}

pub struct Combinations<'a, T, const N: usize> {
    iter: &'a dyn Iterator<Item = T>,
    current_n: usize,
    current_index: usize,
    indexes: [usize; N],
}

impl<'a, T, const N: usize> Combinations<'a, T, N> {
    fn new(iter: &'a dyn Iterator<Item = T>) -> Self {
        Self {
            iter,
            current_n: 0,
            current_index: 0,
            indexes: [0; N],
        }
    }
}

impl<'a, T, const N: usize> Iterator for Combinations<'a, T, N> {
    type Item = &'a [T; N];

    fn next(&mut self) -> Option<&'a [T; N]> {
        todo!();
    }
}
