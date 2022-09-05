//! Solver for Matt Parker's "Wordle5" problem: find five
//! five-letter words from a dictionary that collectively
//! contain 25 of the 26 letters of the English alphabet.
//!
//! The general solution idea is to try to pick words
//! containing 25 letters in order of increasing letter
//! difficulty: number of words containing the letter is a
//! difficulty proxy, with more words being easier.
//!
//! A pruning trick is used to shorten the number of words
//! examined for "easy" letters, by noting that all but one
//! of the previous letters must be used by the time you get
//! to a new letter. (All-but-one because you may have
//! skipped one because 25 of 26 letters will be used.)

use std::collections::HashMap;

/// Type of bitsets of letters, encoded with bit 0 (LSB)
/// representing the presence or absence of 'a', bit 1 'b',
/// and so forth up to bit 25 as 'z'.
type LetterSet = u32;

/// Type of characters, encoded with 0 as 'a', 1 as 'b' and
/// so forth up to 25 as 'z'.
type Char = u32;

/// Given a reference to a `word` from the dictionary, give
/// it back as `Some` `LetterSet` and owned string, if and
/// only if it is a five-letter word with five distinct
/// letters. Otherwise return `None`.
fn five_letter(word: &str) -> Option<(LetterSet, String)> {
    // Check for "weird" words: the dictionaries being
    // used are assumed to have been pre-filtered to
    // remove them.
    if word.chars().any(|c| !c.is_ascii_lowercase()) {
        println!("weird word {word} in dict");
        std::process::exit(1);
    }

    // Filter for words with exactly five letters.
    if word.len() != 5 {
        return None;
    }

    // Compute the `LetterSet` for the word.
    let mut letters: LetterSet = 0;
    for c in word.chars() {
        letters |= 1 << (c as u32 - 'a' as u32);
    }

    // Filter for words that contain five unique letters.
    if letters.count_ones() != 5 {
        return None;
    }

    // Found a valid word.
    Some((letters, word.to_owned()))
}

/// Get the list of dictionaries to use from the command
/// line, and combine their contents to produce a list of
/// five-letter words, each containing five unique letters,
/// and collectively containing only unique `LetterSet`s.
/// This list is returned as a `HashMap` from each word's
/// `LetterSet` to its owned `String` representation.
///
/// `LetterSets` representing sets of of anagrammatic words
/// have a slash-separated `String` representation: for
/// example `"pots/stop/tops/post"`.
fn assemble_dicts(dicts: &[String]) -> HashMap<LetterSet, String> {
    let mut dict: HashMap<LetterSet, String> = HashMap::new();

    // Process each specified dictionary in turn.
    for dname in dicts {
        use std::io::BufRead;

        // Read and filter the dictionary `d`.
        let words = std::fs::File::open(&dname).unwrap_or_else(|e| {
            println!("Could not open dictionary {dname}: {e}");
            std::process::exit(1);
        });
        let words = std::io::BufReader::new(words);

        // Use your words.
        let words = words.lines().filter_map(|s| {
            let s = s.unwrap_or_else(|e| {
                println!("error reading {dname}: {e}");
                std::process::exit(1);
            });
            five_letter(&s)
        });

        // Extend the working dictionary, taking anagrams
        // into account.
        for (w, s) in words {
            dict.entry(w)
                .and_modify(|v| {
                    v.push('/');
                    v.push_str(&s);
                })
                .or_insert(s);
        }
    }
    dict
}

/// Type of all words that contain a given `Char`,
/// represented as the `Char` tag together with a `Vec` of
/// `LetterSet`s for the words.
type LetterGroup = (Char, Vec<LetterSet>);

/// Given a list of `LetterSet`s representing all the words
/// in the dictionary, return a list of `LetterGroup`s.
/// Each `LetterGroup` represents a list of all words from
/// the dictionary containing a given letter. The collective
/// output of this function contains one `LetterGroup` for
/// each letter of the alphabet, ordered by increasing size
/// of the `LetterGroup`. (Same-sized letter groups are
/// arbitrarily ordered.)
///
/// For example, if the input list contained `LetterSet`s representing
/// the words "fishy" and "strip", the output would have a bunch
/// of empty `LetterGroups` for letters not in these words, followed
/// by letter groups notionally like
///
/// ```text
/// ('f', ["fishy"])
/// ('t', ["strip"])
/// ('h', ["fishy"])
/// ('y', ["fishy"])
/// ('p', ["strip"])
/// ('r', ["strip"])
/// ('i', ["fishy", "strip"])
/// ('s', ["fishy", "strip"])
/// ```
fn make_letter_groups(ids: &[LetterSet]) -> Vec<LetterGroup> {
    // Make the letter groups.
    let mut groups = Vec::new();
    for c in 0..26 {
        // Save a letter group for each letter `c`.
        let cs: Vec<LetterSet> = ids
            .iter()
            .filter(|&&id| id & (1 << c) != 0)
            .copied()
            .collect();
        groups.push((c, cs));
    }

    // Sort the letter groups by increasing length of word list.
    groups.sort_unstable_by_key(|(_, cs)| cs.len());

    // Filter each letter group using the idea that only
    // words that contain at most one of the previously-used
    // letters can be used, independent of the solution
    // shape so far.
    let mut seen: LetterSet = 0;
    for (c, words) in &mut groups {
        words.retain(|w| (w & seen).count_ones() < 2);
        seen |= 1 << *c;
    }

    groups
}

#[test]
fn test_make_letter_groups() {
    let groups = make_letter_groups(&[0b11111]);
    for (c, v) in groups {
        if c < 5 {
            assert_eq!(v, [0b11111]);
        } else {
            assert!(v.is_empty());
        }
    }
}

/// Type of problem solutions: a five-element array with
/// each element a `LetterSet` representing a word.
type Solution = [LetterSet; 5];

/// Horribly-named function for actually solving the
/// *Wordle5* problem for a given dictionary. The general
/// strategy is to ensure that exactly one word from the
/// `LetterGroup` at `posn` in `groups` is included in
/// the current solution as represented by `cur`, but with
/// no shared letters in the `Solution`. Exactly one `skip`
/// is allowed, in which a letter is not included in the
/// current solution: this is because there are 26 possible
/// letters and only 25 can be used in the solution. The
/// `count` is the prefix of the current solution that is
/// valid. A set of `seen` letters in the current solution
/// is kept for efficiency.
///
/// A search step as implemented by `solvify()` consists of
/// stepping forward to the next as-yet-unused letter, then
/// trying to include each word in its `LetterGroup` in
/// turn. If a five-word `Solution` is discovered, it is
/// added to the vec of `solns`.
fn solvify(
    groups: &[LetterGroup],
    cur: &mut Solution,
    solns: &mut Vec<Solution>,
    mut posn: usize,
    count: usize,
    seen: LetterSet,
    skipped: bool,
) {
    // If five words have been found, that's a
    // solution. Save it and end this function invocation.
    if count == 5 {
        solns.push(*cur);
        return;
    }

    // Search forward for the next unused letter.
    loop {
        // Ran off the end. End this function invocation.
        if posn >= 26 {
            return;
        }

        // If we've found an unused letter, exit the loop
        // with `posn` set to point to that `LetterGroup`.
        let c = groups[posn].0;
        if seen & (1 << c) == 0 {
            break;
        }

        // Try the next position.
        posn += 1;
    }

    // Try extending the current solution using each word in
    // the current `LetterGroup`.
    for &id in &groups[posn].1 {
        if seen & id != 0 {
            continue;
        }
        cur[count] = id;
        solvify(groups, cur, solns, posn + 1, count + 1, seen | id, skipped);
    }

    // If possible, try extending the current solution by
    // skipping the current `LetterGroup`. This can only
    // happen once for each solution.
    if !skipped {
        solvify(groups, cur, solns, posn + 1, count, seen, true);
    }
}

/// Type of solver functions.
type Solver = fn(&[LetterGroup]) -> Vec<Solution>;

/// Stub to cleanly sequentially invoke `solvify()` and
/// return its solutions.
fn solve_sequential(groups: &[LetterGroup]) -> Vec<Solution> {
    let mut partial = [0; 5];
    let mut solns = Vec::new();
    solvify(groups, &mut partial, &mut solns, 0, 0, 0, false);
    solns
}

/// Stub to invoke `solvify()` using top-level parallelism
/// via `rayon` and return its solutions.
fn solve_rayon(groups: &[LetterGroup]) -> Vec<Solution> {
    use rayon::prelude::*;

    // We will be parallelising over the first group.
    let (_, ws) = &groups[0];

    // Gross hack to handle the skip case at the base level
    // in parallel with the other cases.
    let mut ws = ws.clone();
    ws.push(0);

    // Run the parallel loop.
    ws.as_slice()
        .into_par_iter()
        .map(|&w| {
            let mut partial = [0; 5];
            let mut solns = Vec::new();
            if w != 0 {
                partial[0] = w;
                solvify(groups, &mut partial, &mut solns, 1, 1, w, false);
            } else {
                // Letter skip case.
                solvify(groups, &mut partial, &mut solns, 1, 0, 0, true);
            }
            solns
        })
        .reduce(Vec::new, |mut solns1, solns2| {
            solns1.extend(solns2);
            solns1
        })
}

/// Stub to invoke `solvify()` using top-level parallelism
/// via scoped threads and return its solutions.
fn solve_scoped_threads(groups: &[LetterGroup]) -> Vec<Solution> {
    use std::thread::{scope, ScopedJoinHandle};

    // We will be parallelising over the first group.
    let (_, ws) = &groups[0];

    // Gross hack to handle the skip case at the base level
    // in parallel with the other cases.
    let mut ws = ws.clone();
    ws.push(0);

    // Run the parallel loop.
    scope(move |s| {
        // XXX The collect() at the end here does not seem to me
        // to be needless. I want to ensure that I spawn all the
        // threads before I wait for any of them. Spawning
        // a thread and then joining that thread sequentializes
        // the computation, I think maybe?
        #[allow(clippy::needless_collect)]
        let handles: Vec<ScopedJoinHandle<Vec<Solution>>> = ws
            .into_iter()
            .map(move |w| {
                s.spawn(move || {
                    let mut partial = [0; 5];
                    let mut solns = Vec::new();
                    if w != 0 {
                        partial[0] = w;
                        solvify(groups, &mut partial, &mut solns, 1, 1, w, false);
                    } else {
                        // Letter skip case.
                        solvify(groups, &mut partial, &mut solns, 1, 0, 0, true);
                    }
                    solns
                })
            })
            .collect();

        handles
            .into_iter()
            .fold(Vec::new(), |mut solns, handle| {
                let soln = handle.join().unwrap();
                solns.extend(soln);
                solns
            })
    })
}

/// Solve a Wordle5 problem using dictionaries specified on
/// the command line. Print each solution found.
fn main() {
    // Process arguments.
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return;
    }
    let mut solver: Solver = solve_scoped_threads;
    if args[0].starts_with("--") {
        match args[0].as_str() {
            "--scoped-threads" => solver = solve_scoped_threads,
            "--sequential" => solver = solve_sequential,
            "--rayon" => solver = solve_rayon,
            s => {
                println!("{s}: unknown solver");
                std::process::exit(1);
            }
        }
        args.remove(0);
    }

    // Build supporting data.
    let dict = assemble_dicts(&args);
    let ids: Vec<LetterSet> = dict.keys().copied().collect();
    let groups = make_letter_groups(&ids);

    // Solve the problem and show any resulting solutions.
    for soln in solver(&groups).into_iter() {
        for id in soln {
            print!("{} ", dict[&id]);
        }
        println!();
    }
}
