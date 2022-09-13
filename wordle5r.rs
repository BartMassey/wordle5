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

    fn solve(
        translations: &HashMap<u32, String>,
        words: &Vec<u32>,
        i: usize,
        ws: &mut Vec<u32>,
        seen: u32,
    ) {
        let d = ws.len();
        if d == 5 {
            for w in ws.iter() {
                print!("{} ", translations[w]);
            }
            println!();
        }

        for (j, &w) in words[i..].iter().enumerate() {
            if seen & w != 0 {
                continue;
            }
            ws.push(w);
            solve(translations, words, i + j + 1, ws, w | seen);
            let _ = ws.pop();
        }
    }

    solve(&translations, &dwords, 0, &mut vec![], 0);
}
