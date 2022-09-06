# wordle5: Fast Solver For Wordle5 Puzzle
Bart Massey 2022

## Background

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

## Performance

My solution is blazingly fast, solving the standard problem
in about 12ms of wall clock time on my 12-core Ryzen 9 3900X
desktop using `rayon` parallelism. The solution time is
about 17ms single-threaded.  The comment thread on this
[YouTube video](https://youtu.be/Y37WiO55bxs) seems to be
the source of fastest solutions right now: I'm about a
factor of two faster than the best reported solution, and
still faster when single-threaded.

Flamegraph profiling shows that about two-thirds of the
runtime of the single-threaded version is spent in the
solver proper, so there's still some room for improvement
there, albeit with diminishing returns. For the
multi-threaded versions the solver time appears to be in the
noise: much larger dictionaries would be needed to exercise
many threads sufficiently.

The `main` branch code uses `std::fs::read_to_string()`
followed by line splitting of the string to read the
dictionaries. The branch `bufread` in this repo moves to
using `std::fs::open()` and `std::io::BufRead::lines()`. It
is dramatically slower, taking about 20ms extra just to
process the dictionaries. Rust I/O performance is a bit
wack.

The `main` branch uses a recursive solver. The branch
`nonrecursive-solvify` in this repo makes the solver
iterative using an explicit stack. It is not noticeably
faster currently, but that appears to be a function of the
already-rapid solver time.

When building for best performance, you may want to build a
statically-linked binary for more reproducible best times.
On my box I use the `x86_64-unknown-linux-musl` build target
for this. Note that you definitely want to time the binary:
don't use `cargo run` when timing as it adds major overhead.

I've tried to make my solution clear and readable. Please
see the Rustdoc and source code for details.

## Word Lists

* `words-nyt-wordle.txt` (12945 words): I've taken the
  liberty of combining the NYT Wordle word lists into a
  single file and filtering duplicates. It doesn't change
  performance notably, and it's way more convenient.

* `words-alpha.txt` (15913 words): I've filtered
  `words_alpha.txt` (wherever that came from) to just
  five-letter words and removed all consonant-only words
  (all words contain at least one of *aeiouyw*).

* `words-bart.txt` (17560 words): I've built my own wordlist
  as the union of wordlists in
  [my `wordlists` repo](https://github.com/BartMassey/wordlists),
  filtered the same as `words-alpha.txt`.

## Usage

Build the program with
```
cargo build --release
```

Invoke the program with a list of the dictionary files to be
read. Dictionary files should consist of ASCII lowercase
words, one per line. The easy invocation is

```
cargo run --release words-nyt-wordle.txt
```

You can specify a solver to use with a command-line
argument.

* `--scoped-threads`: This will use the `scoped-threads` multi-threaded solver,
  which is the default.

* `--rayon`: This will use the `rayon` multi-threaded solver, which has similar
  performance to `--scoped-threads`.

* `--sequential`: This will get the sequential
  (single-threaded) solver. It's slower than the
  multi-threaded ones.

So for example
```
cargo run --release -- --sequential words-nyt-wordle.txt
```

## License

This work is made available under the "MIT License."  Please
see the file `LICENSE.txt` in this distribution for license
terms.  The provided dictionaries are used without
permission: no license is provided, express or implied, for
these.
