use std::collections::HashMap;

fn five_letter(word: &str) -> Option<(u32, String)> {
    if word.chars().any(|c| !c.is_ascii_lowercase()) {
        panic!("weird word {word} in dict");
    }
    if word.len() != 5 {
        return None;
    }
    let mut index = 0u32;
    for c in word.chars() {
        index |= 1 << (c as u32 - 'a' as u32);
    }
    if index.count_ones() != 5 {
        return None;
    }
    Some((index, word.to_owned()))
}

fn assemble_dicts() -> HashMap<u32, String> {
    let dicts: Vec<String> = std::env::args().skip(1).collect();
    let mut dict = HashMap::new();
    for d in dicts {
        let text = std::fs::read_to_string(d).unwrap();
        let words = text
            .trim()
            .split('\n')
            .filter_map(five_letter);
        dict.extend(words);
    }
    dict
}


fn letter_freqs(ids: &[u32]) -> [usize; 26] {
    let mut counts = [0; 26];
    for id in ids {
        for c in 0..26 {
            if id & (1 << c) != 0 {
                counts[c] += 1;
            }
        }
    }
    counts
}

fn main() {
    let dict = assemble_dicts();
    let ids: Vec<u32> = dict.keys().copied().collect();
    let freqs = letter_freqs(&ids);
    for (i, f) in freqs.iter().enumerate() {
        let c = char::from_u32(i as u32 + 'a' as u32).unwrap();
        println!("{}: {}", c, f);
    }
}
