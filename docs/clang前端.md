# 

使用clang crate 调用clang 实现前端,方便后端验证


编译clang crate 需要配置llvm库,否则
```
Caused by:
  process didn't exit successfully: `/home/cncsmonster/rust-pro/compiler/compiler/target/debug/build/clang-sys-a5e8f5303c3e90da/build-script-build` (exit status: 101)
  --- stderr
  thread 'main' panicked at /home/cncsmonster/.cargo/registry/src/index.crates.io-6f17d22bba15001f/clang-sys-1.7.0/build/dynamic.rs:206:45:
  called `Result::unwrap()` on an `Err` value: "couldn't find any valid shared libraries matching: ['libclang.so', 'libclang-*.so'], set the `LIBCLANG_PATH` environment variable to a path where one of these files can be found (invalid: [])"
```