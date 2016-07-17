LLVM-rs
=======
[![Crate Version](https://img.shields.io/crates/v/llvm-alt.svg)](https://crates.io/crates/llvm-alt)
[![Crate Downloads](https://img.shields.io/crates/d/llvm-alt.svg)](https://crates.io/crates/llvm-alt)
[![Crate License](https://img.shields.io/crates/l/llvm-alt.svg)](https://crates.io/crates/llvm-alt)
[![Build Status](https://travis-ci.org/TomBebbington/llvm-rs.svg?branch=master)](https://travis-ci.org/TomBebbington/llvm-rs)
[![Dependency Status](https://dependencyci.com/github/TomBebbington/llvm-rs/badge)](https://dependencyci.com/github/TomBebbington/llvm-rs)

This is a library that wraps [LLVM](http://llvm.org) using Rust idioms and the cbox library. There is
[good quality documentation available](https://tombebbington.github.io/llvm-rs/) if you
want to check out the API. It's basically a simplified version of the C++ API which has
[documentation](http://llvm.org/doxygen).

Using in your projects
----------------------
To use this in your project, add the following to your `Cargo.toml`

```toml
[dependencies]
...
llvm-alt = "*"
```

Bugs
----
While using this library, if you ever encounter a segmentation fault or similar unverbose error messages, please file an issue with what code caused the problem. Thanks!
