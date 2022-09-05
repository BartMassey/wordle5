# wordle5: Fast Solver For Wordle5 Puzzle
Bart Massey 2022

YouTuber Matt Parker proposed an interesting problem
inspired by the game *Wordle* a while back: find a set of
five five-letter words from a dictionary that collectively
contain 25 of the 26 letters of the English alphabet.

For a summary history of the problem, see the `README` in
Philip Meier's
[solution repo](https://github.com/pmeier/parker-word-puzzle):
this solution inspired me to create my own solution in Rust;
specifically, I started by responding to this
[Reddit thread](https://www.reddit.com/r/learnrust/comments/x5ykmt/comment/in7l45g/).

My solution is blazingly fast, solving the standard problem
in about 10ms of wall clock time on my 12-core Ryzen 9 3900X
desktop using `rayon` parallelism. The solution time is
about 25ms single-threaded; use `--no-default-features` in
the build for this option.

Flamegraph profiling shows that about two-thirds of the
runtime is spent in the solver proper, so there's still some
room for improvement, albeit with diminishing
returns. Larger dictionaries would load the solver somewhat
harder.

I've tried to make my solution clear and readable. Please
see the Rustdoc and source code for details.

Invoke the program with a list of the dictionary files to be
read. Dictionary files should consist of ASCII lowercase
words, one per line. The standard invocation is

```
cargo run --release wordle-nyt-*.txt
```

This work is made available under the "MIT License."  Please
see the file `LICENSE.txt` in this distribution for license
terms.  The provided dictionaries are used without
permission: no license is provided, express or implied, for
these.
