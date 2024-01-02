use rand;
// 编写单元测试
// 测试f32转化为f64之后再转化为f32是否相等

#[test]
fn test_f32_f64() {
    // 随机生成n个 f32用于测试
    // 测试过就算到10**8 也不会错误
    let n = 1000000;
    let mut f32_vec = Vec::with_capacity(n);
    for _ in 0..n {
        f32_vec.push(rand::random::<f32>());
    }
    // 将f32转化为f64
    let mut f64_vec = Vec::with_capacity(n);
    for f32_val in f32_vec.iter() {
        f64_vec.push(*f32_val as f64);
    }
    // 将f64转化为f32
    let mut f32_vec2 = Vec::with_capacity(n);
    for f64_val in f64_vec.iter() {
        f32_vec2.push(*f64_val as f32);
    }
    // 比较f32_vec和f32_vec2
    for (f32_val, f32_val2) in f32_vec.iter().zip(f32_vec2.iter()) {
        assert_eq!(f32_val, f32_val2);
    }
}
