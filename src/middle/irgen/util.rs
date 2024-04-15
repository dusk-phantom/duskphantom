/// Repeat a vector for `n` times
pub fn repeat_vec<T>(vec: Vec<T>, n: usize) -> Vec<T>
where
    T: Clone,
{
    let mut result = Vec::new();
    for _ in 0..n {
        result.extend(vec.clone());
    }
    result
}
