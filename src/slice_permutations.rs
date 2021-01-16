use super::permutations::LazyPermutationGenerator;
use crate::combinations::LazyCombinationGenerator;
use core::iter::{FusedIterator, Iterator};

#[derive(Clone)]
#[must_use = "iterators do nothing unless consumed"]
pub struct SlicePermutations<'a, T, const K: usize> {
    items: &'a [T],
    comb_gen: LazyCombinationGenerator<K>,
    perm_gen: LazyPermutationGenerator<K>,
}

impl<'a, T, const K: usize> Iterator for SlicePermutations<'a, T, K> {
    type Item = [&'a T; K];

    fn next(&mut self) -> Option<Self::Item> {
        if self.comb_gen.is_done(self.items.len()) {
            None
        } else {
            let items = self.items;
            let comb_indices = self.comb_gen.indices();
            let perm_indices = self.perm_gen.indices();
            let res = crate::make_array(|i| &items[comb_indices[perm_indices[i]]]);
            self.perm_gen.step();
            if self.perm_gen.is_done() {
                // Reset the permutation generator and move to the next combination
                self.perm_gen = LazyPermutationGenerator::new();
                self.comb_gen.step();
            }
            Some(res)
        }
    }
}

impl<'a, T, const K: usize> SlicePermutations<'a, T, K> {
    pub(crate) fn new(items: &'a [T]) -> Self {
        Self {
            items,
            comb_gen: LazyCombinationGenerator::new(),
            perm_gen: LazyPermutationGenerator::new(),
        }
    }
}

impl<T, const K: usize> FusedIterator for SlicePermutations<'_, T, K> {}

#[cfg(test)]
mod test {
    use crate::SliceExt;

    #[test]
    fn order() {
        let mut permutations = [1, 2, 3].permutations();
        assert_eq!(permutations.next(), Some([&1, &2]));
        assert_eq!(permutations.next(), Some([&2, &1]));
        assert_eq!(permutations.next(), Some([&1, &3]));
        assert_eq!(permutations.next(), Some([&3, &1]));
        assert_eq!(permutations.next(), Some([&2, &3]));
        assert_eq!(permutations.next(), Some([&3, &2]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn none_on_size_too_big() {
        let mut permutations = [1].permutations::<2>();
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut permutations = [1, 2, 3, 4].permutations();
        assert_eq!(permutations.next(), Some([]));
        assert_eq!(permutations.next(), None);
        assert_eq!(permutations.next(), None);
    }
}
