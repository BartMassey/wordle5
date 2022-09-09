# wordle5: Fast Solver For Wordle5 Puzzle
Bart Massey 2022

## About This Branch

You should see the `main` branch for the real version of
this program.

This branch is an exercise in building a `no_std` version
for possible startup time win due to not initializing Rust's
runtime. It is an epic failure, showing no speed increase
whatsoever after a massive effort.

You can build the `no_std` version with `cargo +nightly
build --release`.  You must use `--release` because
[Issue #47493](https://github.com/rust-lang/rust/issues/47493). None
of the other Cargo features of this crate work with
`no_std`. You can build with `cargo build --features=std`
and whatever other crate features you like.

## License

This work is made available under the "MIT License."  Please
see the file `LICENSE.txt` in this distribution for license
terms.  The provided dictionaries are used without
permission: no license is provided, express or implied, for
these.
