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
    let mut dwords = Vec::with_capacity(words.len());
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
        dwords.push(b);
    }
    println!("{} translations", translations.len());

    let mut lwords: Vec<(u32, Vec<u32>)> = Vec::with_capacity(26);
    for l in 0..26 {
        let ws: Vec<u32> = translations
            .keys()
            .copied()
            .filter(|&k| k & (1 << l) != 0)
            .collect();
        lwords.push((l, ws));
    }
    lwords.sort_unstable_by_key(|(_, ws)| ws.len());

    let mut seen = 0;
    lwords.retain_mut(|(l, ws)| {
        ws.retain(|w| (w & seen).count_ones() <= 1);
        seen |= 1 << *l;
        !ws.is_empty()
    });

    fn solve(
        translations: &HashMap<u32, String>,
        lwords: &[(u32, Vec<u32>)],
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

        for (j, (l, lws)) in lwords[i..].iter().enumerate() {
            if seen & (1 << *l) != 0 {
                continue;
            }

            for &w in lws.iter() {
                if seen & w != 0 {
                    continue;
                }

                ws.push(w);
                solve(
                    translations,
                    lwords,
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
        solve(&translations, &lwords, 1, &mut vec![w], w, false);
    });
    lwords[1].1.par_iter().for_each(|&w| {
        solve(&translations, &lwords, 2, &mut vec![w], w, true);
    });
}
