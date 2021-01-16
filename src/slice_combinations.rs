use crate::combinations::LazyCombinationGenerator;
use core::iter::{FusedIterator, Iterator};

#[derive(Clone)]
#[must_use = "iterator does nothing unless consumed"]
pub struct SliceCombinations<'a, T, const K: usize> {
    items: &'a [T],
    gen: LazyCombinationGenerator<K>,
}

impl<'a, T, const K: usize> SliceCombinations<'a, T, K> {
    pub(crate) fn new(items: &'a [T]) -> Self {
        Self {
            items,
            gen: LazyCombinationGenerator::new(),
        }
    }
}

impl<'a, T, const K: usize> Iterator for SliceCombinations<'a, T, K> {
    type Item = [&'a T; K];

    fn next(&mut self) -> Option<[&'a T; K]> {
        if self.gen.is_done(self.items.len()) {
            None
        } else {
            let indices = self.gen.indices();
            let res = crate::make_array(|i| &self.items[indices[i]]);
            self.gen.step();
            Some(res)
        }
    }
}

impl<T, const K: usize> FusedIterator for SliceCombinations<'_, T, K> {}

#[cfg(test)]
mod test {
    use crate::SliceExt;

    #[test]
    fn order() {
        let mut combinations = [1, 2, 3, 4, 5].combinations();
        assert_eq!(combinations.next(), Some([&1, &2, &3]));
        assert_eq!(combinations.next(), Some([&1, &2, &4]));
        assert_eq!(combinations.next(), Some([&1, &3, &4]));
        assert_eq!(combinations.next(), Some([&2, &3, &4]));
        assert_eq!(combinations.next(), Some([&1, &2, &5]));
        assert_eq!(combinations.next(), Some([&1, &3, &5]));
        assert_eq!(combinations.next(), Some([&2, &3, &5]));
        assert_eq!(combinations.next(), Some([&1, &4, &5]));
        assert_eq!(combinations.next(), Some([&2, &4, &5]));
        assert_eq!(combinations.next(), Some([&3, &4, &5]));
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn none_on_size_too_big() {
        let mut combinations = [1].combinations::<2>();
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }

    #[test]
    fn empty_arr_on_n_zero() {
        let mut combinations = [1, 2, 3, 4].combinations();
        assert_eq!(combinations.next(), Some([]));
        assert_eq!(combinations.next(), None);
        assert_eq!(combinations.next(), None);
    }
}
