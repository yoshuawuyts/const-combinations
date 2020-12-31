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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.buffer.len();
        if K == 0 {
            (1, Some(1))
        } else if n < K {
            (0, Some(0))
        } else {
            let iter_hint = self.iter.size_hint();
            let lo = n + iter_hint.0;
            let hi = if self.done {
                Some(n)
            } else {
                iter_hint.1.map(|hi| hi + n)
            };
            let lower: Option<usize> = combination_count(lo, K);
            let upper = hi.and_then(|hi| {
                if hi == lo {
                    lower
                } else {
                    combination_count(hi, K)
                }
            });
            (lower.unwrap_or(0), upper)
        }
    }
}

fn combination_count(n: usize, mut k: usize) -> Option<usize> {
    if n < k {
        Some(0)
    } else {
        k = k.min(n - k);
        let mut res: usize = 1;
        for i in 0..k {
            res = res.checked_mul(n - i)?;
            res = res.checked_div(i + 1)?;
        }
        Some(res)
    }
}

#[test]
fn test_combination_count() {
    assert_eq!(combination_count(0, 0), Some(1));
    assert_eq!(combination_count(0, 1), Some(0));

    assert_eq!(combination_count(1, 0), Some(1));
    assert_eq!(combination_count(1, 1), Some(1));
    assert_eq!(combination_count(1, 2), Some(0));

    assert_eq!(combination_count(2, 0), Some(1));
    assert_eq!(combination_count(2, 1), Some(2));
    assert_eq!(combination_count(2, 2), Some(1));
    assert_eq!(combination_count(2, 3), Some(0));

    assert_eq!(combination_count(3, 0), Some(1));
    assert_eq!(combination_count(3, 1), Some(3));
    assert_eq!(combination_count(3, 2), Some(3));
    assert_eq!(combination_count(3, 3), Some(1));
    assert_eq!(combination_count(3, 4), Some(0));
}

fn permutation_count(n: usize, k: usize) -> Option<usize> {
    if n < k {
        Some(0)
    } else {
        let mut res: usize = 1;
        for i in n - k + 1..=n {
            res = res.checked_mul(i)?;
        }
        Some(res)
    }
}

#[test]
fn test_permutation_count() {
    assert_eq!(permutation_count(0, 0), Some(1));
    assert_eq!(permutation_count(0, 1), Some(0));

    assert_eq!(permutation_count(1, 0), Some(1));
    assert_eq!(permutation_count(1, 1), Some(1));
    assert_eq!(permutation_count(1, 2), Some(0));

    assert_eq!(permutation_count(2, 0), Some(1));
    assert_eq!(permutation_count(2, 1), Some(2));
    assert_eq!(permutation_count(2, 2), Some(2));
    assert_eq!(permutation_count(2, 3), Some(0));

    assert_eq!(permutation_count(3, 0), Some(1));
    assert_eq!(permutation_count(3, 1), Some(3));
    assert_eq!(permutation_count(3, 2), Some(6));
    assert_eq!(permutation_count(3, 3), Some(6));
    assert_eq!(permutation_count(3, 4), Some(0));
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.i > 0 {
            (0, Some(0))
        } else {
            let mut returned_count: Option<usize> = Some(if self.first { 0 } else { 1 });
            let mut mult: Option<usize> = Some(1);
            for (i, c) in self.c.iter().enumerate().skip(1) {
                mult = mult.and_then(|mult| mult.checked_mul(i));
                returned_count = returned_count.zip(mult).map(|(rc, mult)| rc + mult * c);
            }
            mult = mult.and_then(|mult| mult.checked_mul(N));
            if let Some((count, rc)) = mult.zip(returned_count) {
                let remaining = count - rc;
                (remaining, Some(remaining))
            } else {
                (0, None)
            }
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let iter_hint = dbg!(self.iter.size_hint());
        let full_perm_count = dbg!(permutation_count(K, K));
        let lower = full_perm_count.map(|cnt| iter_hint.0 * cnt);
        let upper = full_perm_count.and_then(|cnt| iter_hint.1.map(|hi| hi * cnt));
        let perm_hint = dbg!(self
            .perm_iter
            .as_ref()
            .map(|it| it.size_hint())
            .unwrap_or((0, Some(0))));
        (
            lower.unwrap_or(0) + perm_hint.0,
            upper.map(|upper| upper + perm_hint.1.unwrap()),
        )
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

    #[test]
    fn combinations_size_hint() {
        let combinations = (0..5u8).combinations::<2>();
        assert_eq!(combinations.size_hint(), (10, Some(10)));
    }

    #[test]
    fn full_permutations_size_hint() {
        let mut perms = FullPermutations::new([0u8; 5]);
        assert_eq!(perms.size_hint(), (120, Some(120)));
        let mut expexted = 120;
        while let Some(_) = perms.next() {
            expexted -= 1;
            assert_eq!(perms.size_hint(), (expexted, Some(expexted)));
        }
        assert_eq!(perms.size_hint(), (0, Some(0)));
    }

    #[test]
    fn permutations_size_hint() {
        let permutations = (0..5).permutations::<3>();
        assert_eq!(permutations.size_hint(), (60, Some(60)));
        for _ in permutations {}
    }
}
