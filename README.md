# pseudoparker-wordle5
Bart Massey 2022-09-12

This is code for a demo of rewriting Python to Rust.  The
idea is to take a naïve Python Wordle5 solver inspired by
Matt Parker's Python solver and speed it up.

This version uses all the tricks plus for-loop parallelism.
The Python here runs in about 120ms.  The Rust here runs in
about 6ms. The speedup for Rust is about 20×.

This work is made available under the "MIT License". Please
see the file `LICENSE.txt` in this distribution for license
terms.
