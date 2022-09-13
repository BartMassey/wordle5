use std::collections::HashMap;

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

    let mut lwords: HashMap<u32, Vec<u32>> = HashMap::with_capacity(26);
    for l in 0..26 {
        let ws: Vec<u32> = translations
            .keys()
            .copied()
            .filter(|&k| k & (1 << l) != 0)
            .collect();
        lwords.insert(l, ws);
    }

    let mut letters: Vec<(usize, u32)> =
        lwords.iter().map(|(&l, ws)| (ws.len(), l)).collect();
    letters.sort_unstable();
    let letters: Vec<u32> = letters.into_iter().map(|(_, l)| l).collect();

    fn solve(
        translations: &HashMap<u32, String>,
        lwords: &HashMap<u32, Vec<u32>>,
        letters: &[u32],
        i: usize,
        ws: &mut Vec<u32>,
        seen: u32,
        skipped: bool,
    ) {
        let d = ws.len();
        if d == 5 {
            if seen.count_ones() < 25 {
                return;
            }
            for w in ws.iter() {
                print!("{} ", translations[w]);
            }
            println!();
            return;
        }

        for (j, &l) in letters[i..].iter().enumerate() {
            if seen & (1 << l) != 0 {
                continue;
            }

            for &w in lwords[&l].iter() {
                if seen & w != 0 {
                    continue;
                }

                ws.push(w);
                solve(
                    translations,
                    lwords,
                    letters,
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
                    letters,
                    i + j + 1,
                    ws,
                    seen,
                    true,
                );
            }
            return;
        }
    }

    solve(&translations, &lwords, &letters, 0, &mut vec![], 0, false);
}
