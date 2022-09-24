//! Solver for Matt Parker's "Wordle5" problem: find five
//! five-letter words from a dictionary that collectively
//! contain 25 of the 26 letters of the English alphabet.
//!
//! The general solution idea is to try to pick words
//! containing 25 letters in order of increasing letter
//! difficulty: number of words containing the letter is a
//! difficulty proxy, with more words being easier.
//!
//! See this crate's `README` for a detailed description
//! of the algorithm and its implementation.

use std::collections::HashMap as Map;

#[cfg(feature = "instrument")]
mod instrument {
    pub use once_cell::sync::Lazy;
    pub use std::sync::atomic::{
        AtomicUsize,
        Ordering::{AcqRel, Acquire, Relaxed, Release},
    };
    pub static NODES: Lazy<[AtomicUsize; 6]> =
        Lazy::new(|| std::array::from_fn(|_| AtomicUsize::new(0)));
}
#[cfg(feature = "instrument")]
use instrument::*;

#[cfg(any(feature = "timing", feature = "full-timing"))]
use howlong::HighResolutionTimer;

/// Type of bitsets of letters, encoded with bit 0 (LSB)
/// representing the presence or absence of 'a', bit 1 'b',
/// and so forth up to bit 25 as 'z'.
type LetterSet = u32;

#[cfg(feature = "instrument")]
fn letterset_string(cs: LetterSet) -> String {
    let mut result = String::with_capacity(26);
    for i in 0..26 {
        if cs & (1 << i) != 0 {
            result.push(char::try_from('a' as u32 + i).unwrap());
        }
    }
    result
}

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
/// This list is returned as a map from each word's
/// `LetterSet` to its owned `String` representation.
///
/// `LetterSets` representing sets of of anagrammatic words
/// have a slash-separated `String` representation: for
/// example `"pots/stop/tops/post"`.
fn assemble_dicts(dicts: &[String]) -> Map<LetterSet, String> {
    let mut dict: Map<LetterSet, String> = Map::new();

    // Process each specified dictionary in turn.
    for d in dicts {
        // Read and filter the dictionary `d`.
        let text = std::fs::read_to_string(d).unwrap_or_else(|e| {
            println!("Could not read dictionary {d}: {e}");
            std::process::exit(1);
        });
        let words = text.trim().split('\n').filter_map(five_letter);

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
struct LetterGroup {
    letter: Char,
    words: Vec<LetterSet>,
}

/// Type of Wordle5's "search space".
struct LetterSpace {
    groups: Vec<LetterGroup>,
    pseudovowels: LetterSet,
    global_pseudovowels: LetterSet,
}

/// Given a list of `LetterSet`s representing all the words
/// in the dictionary, return a list of `LetterGroup`s and a
/// set of pseudovowels.
///
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
///
/// This isn't entirely accurate, though. Each `LetterGroup`
/// word list is pruned of words containing two or more
/// letters indexing any previous group.  It is also pruned
/// of words containing excess pseudovowels. This pruning
/// allows faster search.
///
/// We also return a `LetterSet` of "pseudovowels".  This is
/// a set of letters such that at least one of them must be
/// in any word not containing the key letter from any
/// preceding group. This is another pruning opportunity.
fn make_letter_space(dwords: &[LetterSet]) -> LetterSpace {
    // Make the letter groups.
    let mut groups = Vec::new();
    for letter in 0..26 {
        // Save a letter group for each letter `c`.
        let words: Vec<LetterSet> = dwords
            .iter()
            .filter(|&&word| word & (1 << letter) != 0)
            .copied()
            .collect();
        groups.push(LetterGroup { letter, words });
    }

    // Sort the letter groups by increasing length of word list.
    groups.sort_unstable_by_key(|LetterGroup { words, .. }| words.len());

    // Filter each letter group using the idea that only
    // words that contain at most one of the previously-used
    // letters can be used, independent of the solution
    // shape so far.
    let mut seen: LetterSet = 0;
    for g in &mut groups {
        g.words.retain(|w| (w & seen).count_ones() < 2);
        seen |= 1 << g.letter;
    }

    // Check a set of pseudovowels for validity.
    let is_pseudovowels = |pseudovowels, words| {
        for &w in words {
            if w & pseudovowels == 0 {
                return false;
            }
        }
        true
    };

    // Calculate pseudovowels or 0 if no pseudovowels are found.
    let find_pseudovowels = |global| {
        // Calculate pseudovowels.
        let mut pv_set = 0;
        let mut pv_letters = Vec::with_capacity(26);
        if global {
            let mut letter_freqs: Vec<(usize, Char)> = (0..26)
                .map(|l| {
                    let count = dwords.iter().filter(|&&w| w & (1 << l) != 0).count();
                    (count, l)
                })
                .collect();
            letter_freqs.sort_unstable_by_key(|(c, _)| std::cmp::Reverse(*c));

            // Note: This loop is guaranteed to terminate
            // early with a set of pseudovowels, which may
            // be the whole alphabet.
            for (_, l) in letter_freqs {
                pv_set |= 1 << l;
                pv_letters.push(l);
                if is_pseudovowels(pv_set, dwords) {
                    break;
                }
            }
        } else {
            // Find the most frequent letter in the current list.
            let next_letter = |words: &[LetterSet]| {
                (0..26)
                    .map(|l| {
                        let count = words.iter().filter(|&&w| w & (1 << l) != 0).count();
                        (count, l)
                    })
                    .max()
                    .unwrap()
                    .1
            };

            let mut words = dwords.to_vec();
            // Note: This loop is guaranteed to terminate
            // with a set of pseudovowels, which may be the
            // whole alphabet.
            while !words.is_empty() {
                let letter = next_letter(&words);
                pv_letters.push(letter);
                let c = 1 << letter;
                pv_set |= c;
                words.retain(|&w| w & c == 0);
            }
        }
        if pv_set.count_ones() >= 26 {
            return 0;
        }

        // Prune pseudovowels greedily until no further prune works.
        if !global {
            // Try to remove an extra element from pseudovowels.
            let reduce_pv = |pv_letters: &[Char], pv_set: LetterSet| {
                for &l in pv_letters.iter() {
                    let pv_reduced = pv_set & !(1 << l);
                    if is_pseudovowels(pv_reduced, dwords) {
                        return Some(l);
                    }
                }
                None
            };

            pv_letters.reverse();
            while let Some(letter) = reduce_pv(&pv_letters, pv_set) {
                pv_letters.retain(|&l| l != letter);
                pv_set &= !(1 << letter);
            }
        }

        pv_set
    };

    let pseudovowels = find_pseudovowels(false);
    #[cfg(feature = "instrument")]
    eprintln!("pseudovowels: {}", letterset_string(pseudovowels));

    let mut global_pseudovowels = find_pseudovowels(true);
    if global_pseudovowels == pseudovowels {
        global_pseudovowels = 0;
    }
    #[cfg(feature = "instrument")]
    eprintln!(
        "global pseudovowels: {}",
        letterset_string(global_pseudovowels)
    );

    // Filter groups for legal pseudovowel usage.
    let ps = pseudovowels;
    let gps = global_pseudovowels;
    let nps = ps.count_ones();
    let ngps = gps.count_ones();
    for g in &mut groups {
        g.words.retain(move |w| {
            let pkeep = (ps & w).count_ones() + 5 <= nps;
            let gpkeep = (gps & w).count_ones() + 5 <= ngps;
            pkeep && gpkeep
        });
    }

    LetterSpace {
        groups,
        pseudovowels,
        global_pseudovowels,
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
#[allow(clippy::too_many_arguments)]
fn solvify(
    space: &LetterSpace,
    cur: &mut Solution,
    solns: &mut Vec<Solution>,
    mut posn: usize,
    count: usize,
    seen: LetterSet,
    skipped: bool,
) {
    #[cfg(feature = "instrument")]
    NODES[count].fetch_add(1, AcqRel);

    // If five words have been found, that's a
    // solution. Save it and end this function invocation.
    if count == 5 {
        solns.push(*cur);
        return;
    }

    // Search forward for the next unused letter.
    let groups = &space.groups;
    loop {
        // Ran off the end. End this function invocation.
        if posn >= 26 {
            return;
        }

        // If we've found an unused letter, exit the loop
        // with `posn` set to point to that `LetterGroup`.
        let c = groups[posn].letter;
        if seen & (1 << c) == 0 {
            break;
        }

        // Try the next position.
        posn += 1;
    }

    // Try extending the current solution using each word in
    // the current `LetterGroup`.
    let pvs = space.pseudovowels;
    let npvs = pvs.count_ones() as usize;
    let gpvs = space.global_pseudovowels;
    let ngpvs = gpvs.count_ones() as usize;
    for &word in &groups[posn].words {
        // Check for letter re-use.
        if seen & word != 0 {
            continue;
        }

        // Check for pseudovowel over-use.
        let overused = |p: LetterSet, n: usize| {
            if p != 0 {
                let p_used = ((seen | word) & p).count_ones() as usize;
                if n - p_used < 4 - count {
                    return true;
                }
            }
            false
        };
        if overused(pvs, npvs) || overused(gpvs, ngpvs) {
            continue;
        }

        // Found a partial solution.
        cur[count] = word;
        solvify(space, cur, solns, posn + 1, count + 1, seen | word, skipped);
    }

    // If possible, try extending the current solution by
    // skipping the current `LetterGroup`. This can only
    // happen once for each solution.
    if !skipped {
        solvify(space, cur, solns, posn + 1, count, seen, true);
    }
}

/// Stub to cleanly sequentially invoke `solvify()` and
/// return its solutions.
fn solve_sequential(space: &LetterSpace) -> Vec<Solution> {
    let mut partial = [0; 5];
    let mut solns = Vec::new();
    solvify(space, &mut partial, &mut solns, 0, 0, 0, false);
    solns
}

/// Solve a Wordle5 problem using dictionaries specified on
/// the command line. Print each solution found.
fn main() {
    #[cfg(feature = "full-timing")]
    let full_timer = HighResolutionTimer::new();

    #[cfg(feature = "timing")]
    let timer = HighResolutionTimer::new();

    // Build supporting data.
    let dicts: Vec<String> = std::env::args().skip(1).collect();
    if dicts.is_empty() {
        eprintln!("no dictionaries");
        std::process::exit(1);
    }
    let dict = assemble_dicts(&dicts);
    let ids: Vec<LetterSet> = dict.keys().copied().collect();
    let space = make_letter_space(&ids);

    #[cfg(feature = "timing")]
    eprintln!("init time: {:?}", timer.elapsed());

    #[cfg(feature = "timing")]
    let timer = HighResolutionTimer::new();

    // Solve the problem.
    let solve = solve_sequential(&space);

    #[cfg(feature = "timing")]
    eprintln!("solve time: {:?}", timer.elapsed());

    #[cfg(feature = "timing")]
    let timer = HighResolutionTimer::new();

    // Show any resulting solutions.
    for soln in solve.into_iter() {
        for id in soln {
            print!("{} ", dict[&id]);
        }
        println!();
    }

    #[cfg(feature = "timing")]
    eprintln!("display time: {:?}", timer.elapsed());

    #[cfg(feature = "full-timing")]
    eprintln!("time: {:?}", full_timer.elapsed());

    #[cfg(feature = "instrument")]
    {
        let total: usize = NODES.iter().map(|c| c.load(Acquire)).sum();
        eprintln!("nodes: {total}");
        for (depth, count) in NODES.iter().enumerate() {
            eprintln!("    {depth}: {}", count.load(Acquire));
        }
    }
}
