// 实现一个fprintln宏用于写入文件
#[macro_export]
macro_rules! fprint {
    ($file:expr, $($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let s=format!($($arg)*);
            $crate::utils::log::must_write($file, &s,false);
        }
    };
    ($file:expr;$mode:literal;$($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let s=format!($($arg)*);
            let append=$mode=='a';
            $crate::utils::log::must_write($file, &s,append);
        }
    };
}
#[macro_export]
macro_rules! fprintln {
    ($file:expr, $($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let mut s=format!($($arg)*);
            s.push('\n');
            $crate::utils::log::must_write($file, &s,false);
        }
    };
    ($file:expr;$mode:literal;$($arg:tt)*) => {
        #[cfg(feature = "log_enabled")]
        {
            let mut s=format!($($arg)*);
            s.push('\n');
            let append=$mode=='a';
            $crate::utils::log::must_write($file, &s,append);
        }
    };
}

#[allow(unused)]
pub fn must_write(path: &str, content: &str, append: bool) {
    use std::fs::OpenOptions;
    use std::io::Write;
    // if dir not exists, create it
    let path = std::path::Path::new(path);
    if let Some(dir) = path.parent() {
        if !dir.exists() {
            std::fs::create_dir_all(dir).unwrap();
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(path)
        .unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
}