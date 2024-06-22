
#[macro_export]
macro_rules! context {
    () => {
        concat!(file!(), ":", line!(), ":", column!())
    };
}
