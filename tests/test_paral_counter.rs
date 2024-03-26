extern crate compiler;
#[cfg(test)]
pub mod tests {
    use compiler::utils::paral_counter::ParalCounter;
    use std::collections::HashSet;
    use std::sync::Arc;
    #[test]
    fn test_paral_counter() {
        let start = 0;
        let end = 1000_000;
        let counter = ParalCounter::new(0, end);
        // 开启多个线程,每个线程分配id,收集最终id
        let mut ids: HashSet<usize> = HashSet::with_capacity(end - start + 1);
        let mut handles = Vec::new();
        let counter = Arc::new(counter);
        for _i in 0..8 {
            let counter = Arc::clone(&counter);
            let handle = std::thread::spawn(move || {
                let mut ids = HashSet::new();
                while let Some(id) = counter.get_id() {
                    ids.insert(id);
                }
                return ids;
            });
            handles.push(handle);
        }
        for handle in handles {
            let par_ids = handle.join();
            if let Ok(par_ids) = par_ids {
                ids.extend(par_ids.iter().clone())
            } else if let Err(e) = par_ids {
                panic!("thread panic: {:?}", e);
            }
        }
        for i in start..end {
            assert!(ids.contains(&i));
        }
    }
}