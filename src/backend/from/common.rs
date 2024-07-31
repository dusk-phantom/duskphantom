#[derive(Debug, Clone)]
pub enum Dimension {
    One(usize),
    Mixture(Vec<Dimension>),
}

impl Dimension {
    pub fn new(size: usize) -> Self {
        Dimension::One(size)
    }
    pub fn new_mixture(dims: Vec<Dimension>) -> Self {
        Dimension::Mixture(dims)
    }
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
            Dimension::Mixture(dims) => dims.iter().for_each(|dim| subs.extend(dim.iter_subs())),
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
}
