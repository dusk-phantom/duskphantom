/// 表示值类型
/// 如果是函数，则表示该函数的返回值类型。
/// 如果是指令，若为非指针指令，则为该指令代表的值，若为指针或者数组，则表示该指令指向的值的类型。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Void,
    Int,
    Float,
}
