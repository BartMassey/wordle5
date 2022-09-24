use std::collections::HashMap;

use rayon::prelude::*;

fn main() {
    // Process arguments.
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 2 {
        println!("usage: <dict>");
        std::process::exit(1);
    }

    // Read dictionary.
    let filename = &argv[1];
    let words: String = std::fs::read_to_string(filename).unwrap_or_else(|e| {
        println!("{filename}: cannot read: {e}");
        std::process::exit(1);
    });
    let words: Vec<&str> = words
        .trim()
        .split('\n')
        .collect();
    println!("{} words", words.len());

    fn bits(w: &str) -> u32 {
        let mut b = 0;
        for c in w.chars() {
            b |= 1 << (c as u32 - 'a' as u32);
        }
        b
    }

    let mut translations: HashMap<u32, String> =
        HashMap::with_capacity(words.len());
    let mut wsets = Vec::with_capacity(words.len());
    for w in words {
        let b = bits(w);
        if b.count_ones() < 5 {
            continue;
        }
        // Deal with anagrams.
        if let Some(tw) = translations.get_mut(&b) {
            tw.push('/');
            tw.push_str(w);
            continue;
        }
        translations.insert(b, w.to_string());
        wsets.push(b);
    }
    println!("{} translations", translations.len());

    let mut lwords: Vec<(u32, Vec<u32>)> = Vec::with_capacity(26);
    for l in 0..26 {
        let ws: Vec<u32> = wsets
            .iter()
            .copied()
            .filter(|&k| k & (1 << l) != 0)
            .collect();
        lwords.push((l, ws));
    }
    lwords.sort_unstable_by_key(|(_, ws)| ws.len());

    fn to_chars(letters: &[u32]) -> String {
        letters
            .iter()
            .copied()
            .map(|l| {
                char::from_u32(l + 'a' as u32).unwrap()
            })
            .collect()
    }

    let mut vowels = 0;
    let mut vowel_letters = Vec::with_capacity(26);
    let mut remaining = wsets.clone();
    for (l, _) in lwords.iter().rev() {
        vowel_letters.push(*l);
        vowels |= 1 << *l;
        remaining.retain(|w| (w & vowels) == 0);
        if remaining.is_empty() {
            break;
        }
    }

    let mut gvowels = 0;
    let mut gvowel_letters = Vec::with_capacity(26);
    let mut glwords = lwords.clone();
    while !glwords.is_empty() {
        let (l, _) = glwords.iter_mut().max_by_key(|(_, ws)| ws.len()).unwrap();
        gvowel_letters.push(*l);
        gvowels |= 1 << *l;
        glwords.retain_mut(|(_, ws)| {
            ws.retain(|w| (w & gvowels) == 0);
            !ws.is_empty()
        });
    }

    let prune = |vowel_letters: &[u32], vowels: u32| -> (Vec<u32>, u32) {
        let mut cvowels = vowels;
        let mut candidates = vowel_letters.iter().copied().rev();
        let mut cletters = Vec::with_capacity(vowel_letters.len());
        cletters.push(candidates.next().unwrap());
        for l in candidates {
            cvowels &= !(1 << l);
            for &w in wsets.iter() {
                if (w & cvowels) == 0 {
                    cvowels |= 1 << l;
                    cletters.push(l);
                    break;
                }
            }
        }
        (cletters.into_iter().rev().collect(), cvowels)
    };

    let (vowel_letters, vowels) = prune(&vowel_letters, vowels);
    println!("pseudovowels: {}", to_chars(&vowel_letters));
    let (gvowel_letters, _gvowels) = prune(&gvowel_letters, gvowels);
    println!("greedy pseudovowels: {}", to_chars(&gvowel_letters));

    let mut seen = 0;
    lwords.retain_mut(|(l, ws)| {
        ws.retain(|w| (w & seen).count_ones() <= 1);
        seen |= 1 << *l;
        !ws.is_empty()
    });

    fn solve(
        translations: &HashMap<u32, String>,
        lwords: &[(u32, Vec<u32>)],
        pvowels@(vowels, gvowels): (u32, u32),
        i: usize,
        ws: &mut Vec<u32>,
        seen: u32,
        skipped: bool,
    ) {
        let d = ws.len();
        if d == 5 {
            for w in ws.iter() {
                print!("{} ", translations[w]);
            }
            println!();
            return;
        }

        let nvowels = vowels.count_ones();
        let ngvowels = gvowels.count_ones();
        for (j, (l, lws)) in lwords[i..].iter().enumerate() {
            if seen & (1 << *l) != 0 {
                continue;
            }

            for &w in lws.iter() {
                if seen & w != 0 {
                    continue;
                }

                if nvowels - (vowels & (w | seen)).count_ones() < 4 - d as u32 {
                    continue;
                }

                if ngvowels - (gvowels & (w | seen)).count_ones() < 4 - d as u32 {
                    continue;
                }

                ws.push(w);
                solve(
                    translations,
                    lwords,
                    pvowels,
                    i + j + 1,
                    ws,
                    w | seen,
                    skipped,
                );
                let _ = ws.pop();
            }

            if !skipped {
                solve(
                    translations,
                    lwords,
                    pvowels,
                    i + j + 1,
                    ws,
                    seen,
                    true,
                );
            }
            return;
        }
    }

    lwords[0].1.par_iter().for_each(|&w| {
        solve(&translations, &lwords, (vowels, gvowels), 1, &mut vec![w], w, false);
    });
    lwords[1].1.par_iter().for_each(|&w| {
        solve(&translations, &lwords, (vowels, gvowels), 2, &mut vec![w], w, true);
    });
}
