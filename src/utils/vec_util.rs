use anyhow::Result;

pub fn insert_vec<T: Clone>(vec: &mut Vec<T>, mut to_insert: Vec<(usize, T)>) -> Result<()> {
    if to_insert.is_empty() {
        return Ok(());
    }
    to_insert.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(to_insert[0].0 < vec.len());
    let mut new_vec = Vec::with_capacity(vec.len() + to_insert.len());
    for (i, item) in vec.iter().enumerate() {
        while let Some((index, _)) = to_insert.first() {
            if index == &i {
                let (_, to_insert_item) = to_insert.remove(0);
                new_vec.push(to_insert_item);
            } else {
                break;
            }
        }
        new_vec.push(item.clone());
    }
    *vec = new_vec;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_vec() {
        let mut vec = vec![1, 2, 3, 4, 5];
        insert_vec(&mut vec, vec![(0, 0), (2, 2), (4, 4)].into_iter().collect()).unwrap();
        assert_eq!(vec, vec![0, 1, 2, 2, 3, 4, 4, 5]);
    }
}
