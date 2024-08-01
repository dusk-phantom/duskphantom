#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dimension {
    One(usize),
    Mixture(Vec<Dimension>),
}

impl Dimension {
    pub fn size(&self) -> usize {
        match self {
            Dimension::One(size) => *size,
            Dimension::Mixture(dims) => dims.iter().map(|dim| dim.size()).sum(),
        }
    }
    pub fn iter_subs(&self) -> std::vec::IntoIter<&Dimension> {
        let mut subs = vec![];
        match self {
            Dimension::One(_) => {}
            Dimension::Mixture(dims) => dims.iter().for_each(|dim| subs.push(dim)),
        }
        subs.into_iter()
    }

    pub fn count_idx(&self, idxs: &[usize]) -> usize {
        let mut ret = 0;
        let mut it = vec![self].into_iter();
        for idx in idxs {
            if let Some(dim) = it.next() {
                ret += dim.size() * idx;
                it = dim.iter_subs();
            } else {
                break;
            }
        }
        ret
    }

    // check if each dimension in the same layer has the same type,thus is a array like [0;n] or [[0;n];m]
    pub fn is_array_like(&self) -> bool {
        match self {
            Dimension::One(_) => true,
            Dimension::Mixture(dims) => {
                let mut it = dims.iter();
                if let Some(first) = it.next() {
                    let first = first.size();
                    it.all(|dim| dim.size() == first)
                } else {
                    true
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_size() {
        let l3 = Dimension::One(3);
        assert_eq!(l3.size(), 3);
        let l2 = Dimension::Mixture(vec![Dimension::One(2), Dimension::One(3)]);
        assert_eq!(l2.size(), 5);
    }

    #[test]
    fn test_iter_subs() {
        let l3 = Dimension::One(3);
        let mut it = l3.iter_subs();
        assert_eq!(it.next(), None);
        let l2 = Dimension::Mixture(vec![Dimension::One(2), Dimension::One(3)]);
        let mut it = l2.iter_subs();
        assert_eq!(it.next(), Some(&Dimension::One(2)));
        assert_eq!(it.next(), Some(&Dimension::One(3)));
        assert_eq!(it.next(), None);
    }
}
