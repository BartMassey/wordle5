# pseudoparker-wordle5
Bart Massey 2022-09-12

This is code for a demo of rewriting Python to Rust.  The
idea is to take a naïve Python Wordle5 solver inspired by
Matt Parker's Python solver and speed it up.

This version uses greedy pruning to produce an extra set of
7 pseudovowels. This set is used along with the original set
of 9 pseudovowels in vowel pruning.  The Python here runs in
about 373ms.  The Rust here runs in about 9ms. The speedup
for Rust is about 40×.

The timing data is quite suspect at this point: have
confirmed that it is not being measured
accurately. Different tooling is needed.

This work is made available under the "MIT License". Please
see the file `LICENSE.txt` in this distribution for license
terms.
