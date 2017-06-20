# libflowd-rs

Parse and marshal [libflowd v2 frames](https://github.com/ERnsTL/flowd), enabling the creation of ```flowd``` components in Rust.


## Benchmarks

Run them with:

```
rustc -C opt-level=3 src/lib.rs --test -o bench && ./bench --bench
```
