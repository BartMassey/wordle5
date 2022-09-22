# pseudoparker-wordle5
Bart Massey 2022-09-12

This is code for a demo of rewriting Python to Rust.  The
idea is to take a naïve Python Wordle5 solver inspired by
Matt Parker's Python solver and speed it up.

This version tries to "prune" the pseudovowels themselves to
reduce the set size. It produces no gain on the
currently-computed pseudovowels. The Python here runs in
about 710ms.  The Rust here runs in about 11ms. The speedup
for Rust is about 65×.

The timing data is quite suspect at this point: have
confirmed that it is not being measured
accurately. Different tooling is needed.

This work is made available under the "MIT License". Please
see the file `LICENSE.txt` in this distribution for license
terms.
