use std::iter::Iterator;
use std::mem::MaybeUninit;

/// An iterator that returns k-length combinations of values from `iter`.
///
/// This `struct` is created by the [`combinations`] method on [`IterExt`]. See its
/// documentation for more.
///
/// [`combinations`]: super::IterExt::combinations
/// [`IterExt`]: super::IterExt
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Combinations<I, const K: usize>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: I,
    buffer: Vec<I::Item>,
    indices: [usize; K],
    first: bool,
}

impl<I, const K: usize> Combinations<I, K>
where
    I: Iterator,
    I::Item: Clone,
{
    pub(crate) fn new(iter: I) -> Self {
        // Prepare the indices.
        let mut indices = [0; K];
        // NOTE: this clippy attribute can be removed once we can `collect` into `[usize; K]`.
        #[allow(clippy::clippy::needless_range_loop)]
        for i in 0..K {
            indices[i] = i;
        }

        Self {
            buffer: Vec::new(),
            indices,
            first: true,
            iter,
        }
    }
}

impl<I, const K: usize> Iterator for Combinations<I, K>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = [I::Item; K];

    fn next(&mut self) -> Option<[I::Item; K]> {
        if self.first {
            // Fill the buffer for the first combination
            self.buffer.reserve(K - self.buffer.len());
            while self.buffer.len() < K {
                match self.iter.next() {
                    Some(x) => self.buffer.push(x),
                    None => return None,
                }
            }
            self.first = false;
        } else if K == 0 {
            // This check is separated, because in case of K == 0 we still
            // need to return a single empty array before returning None.
            return None;
        } else {
            // Check if we need to consume more from the iterator
            if self.indices[0] == self.buffer.len() - K {
                // Exhausted all combinations in the current buffer
                match self.iter.next() {
                    Some(x) => self.buffer.push(x),
                    None => return None,
                }
            }

            let mut i = 0;
            // Reset consecutive indices
            while i < K - 1 && self.indices[i] + 1 == self.indices[i + 1] {
                self.indices[i] = i;
                i += 1;
            }
            // Increment the last consecutive index
            self.indices[i] += 1;
        }

        // Create the result array based on the indices
        let mut out: [MaybeUninit<I::Item>; K] = MaybeUninit::uninit_array();
        self.indices.iter().enumerate().for_each(|(oi, i)| {
            out[oi] = MaybeUninit::new(self.buffer[*i].clone());
        });
        Some(unsafe { out.as_ptr().cast::<[I::Item; K]>().read() })
    }
}

#[cfg(test)]
mod test {
    use crate::IterExt;

    #[test]
    fn order() {
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
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn none_on_size_too_big() {
        let mut combinations = (1..2).combinations::<2>();
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut combinations = (1..5).combinations();
        assert_eq!(combinations.next(), Some([]));
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn resume_after_none() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut combinations = receiver.try_iter().combinations();
        assert_eq!(combinations.next(), None);

        sender.send(1).unwrap();
        assert_eq!(combinations.next(), None);

        sender.send(2).unwrap();
        assert_eq!(combinations.next(), None);

        sender.send(3).unwrap();
        assert_eq!(combinations.next(), Some([1, 2, 3]));
        assert_eq!(combinations.next(), None);

        sender.send(4).unwrap();
        assert_eq!(combinations.next(), Some([1, 2, 4]));
        assert_eq!(combinations.next(), Some([1, 3, 4]));
        assert_eq!(combinations.next(), Some([2, 3, 4]));
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }
}
