pub enum MaybeOwned<'a, T: 'a> {
    Owned(T),
    Borrowed(&'a T),
}

impl<T> From<T> for MaybeOwned<'_, T> {
    fn from(t: T) -> Self {
        MaybeOwned::Owned(t)
    }
}

impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    fn from(t: &'a T) -> Self {
        MaybeOwned::Borrowed(t)
    }
}

impl<'a, T> AsRef<T> for MaybeOwned<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            MaybeOwned::Owned(t) => t,
            MaybeOwned::Borrowed(t) => t,
        }
    }
}
