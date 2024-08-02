// 实现一个fprintln宏用于写入文件
#[macro_export]
macro_rules! fprint {
    ($file:expr, $($arg:tt)*) => {
        {
            let s=format!($($arg)*);
            std::fs::write($file, s.as_bytes())
        }
    };
}
#[macro_export]
macro_rules! fprintln {
    ($file:expr, $($arg:tt)*) => {
        {
            let mut s=String::new();
            let a=format!($($arg)*);
            s.push_str(&a);
            s.push('\n');
            std::fs::write($file, s.as_bytes())
        }
    };
}
