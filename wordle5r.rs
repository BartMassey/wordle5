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


    let make_pseudovowels_lazy = || {
        let make = || {
            let ls = letters.iter().rev().cloned();
            let mut ws: Vec<u32> = wsets.to_vec();
            let mut pvs = 0;
            let mut pvls = Vec::new();
            for l in ls {
                pvls.push(l);
                let x = 1 << l;
                pvs |= x;
                ws.retain(|&w| (w & x) == 0);
                if ws.is_empty() {
                    return (pvs, pvls);
                }
            }
            panic!("ran out of letters for pseudovowels");
        };

        let reduce = |pvs: u32, pvls: &mut Vec<u32>| {
            for (i, l) in pvls.clone().into_iter().enumerate().rev() {
                let new_pvs = pvs & !(1 << l);
                if wsets.iter().all(|&w| (w & new_pvs) != 0) {
                    pvls.remove(i);
                    return new_pvs;
                }
            }
            pvs
        };

        let (mut pvs, mut pvls) = make();
        loop {
            let new_pvs = reduce(pvs, &mut pvls);
            if new_pvs == pvs {
                break;
            }
            pvs = new_pvs;
        }
        let pvls: String = pvls
            .into_iter()
            .map(|l| char::try_from(l + 'a' as u32).unwrap())
            .collect();
        println!("pseudovowels: {pvls}");
        pvs
    };

    let pvs_lazy = make_pseudovowels_lazy();

    #[allow(clippy::too_many_arguments)]
    fn solve(
        translations: &HashMap<u32, String>,
        lwords: &HashMap<u32, Vec<u32>>,
        letters: &[u32],
        pvs_lazy: u32,
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

        let npvs_lazy = pvs_lazy.count_ones();
        for (j, &l) in letters[i..].iter().enumerate() {
            if (seen & (1 << l)) != 0 {
                continue;
            }

            for &w in lwords[&l].iter() {
                if (w & seen) != 0 {
                    continue;
                }

                let pvl = (w | seen) & pvs_lazy;
                let npvl = pvl.count_ones();
                if npvl + (4 - d as u32) > npvs_lazy {
                    continue;
                }

                ws.push(w);
                solve(
                    translations,
                    lwords,
                    letters,
                    pvs_lazy,
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
                    pvs_lazy,
                    i + j + 1,
                    ws,
                    seen,
                    true,
                );
            }

            return;
        }
    }

    solve(
        &translations,
        &lwords,
        &letters,
        pvs_lazy,
        0,
        &mut vec![],
        0,
        false,
    );
}
