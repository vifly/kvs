# kvs
This is my practice project to learn Rust from PingCAP's [talent-plan](https://github.com/pingcap/talent-plan/blob/master/courses/rust/README.md).

## Differences from the original project
1. Original dependencies version is too old, so I upgrade some lib version.
2. To use argh instead of clap, I change CLI args position in test files. For example, `kvs-client set key1 value1 --addr 127.0.0.1:4003` -> `kvs-client --addr 127.0.0.1:4003 set key1 value1`.
