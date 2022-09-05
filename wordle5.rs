use std::collections::HashMap;

use ordered_float::NotNan;

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
        for (c, n) in counts.iter_mut().enumerate() {
            if id & (1 << c) != 0 {
                *n += 1;
            }
        }
    }
    counts
}

fn score_id(freqs: &[usize; 26], id: u32) -> NotNan<f32> {
    let mut sum = 0.0;
    for (c, &f) in freqs.iter().enumerate() {
        if id & (1 << c) != 0 {
            sum += f32::log2(f as f32);
        }
    }
    NotNan::new(sum).unwrap()
}

#[test]
fn test_score_id() {
    let mut freqs = [0; 26];
    for i in 0..5 {
        freqs[i] = 2;
    }
    let id = five_letter("abcde").unwrap().0;
    assert_eq!(id, 0b11111);
    assert_eq!(score_id(&freqs, id).into_inner(), 5.0);
}

fn main() {
    let dict = assemble_dicts();
    let mut ids: Vec<u32> = dict.keys().copied().collect();
    let freqs = letter_freqs(&ids);
    let scores: HashMap<u32, NotNan<f32>> = ids
        .iter()
        .map(|&id| (id, score_id(&freqs, id)))
        .collect();
    ids.sort_unstable_by_key(|id| scores[id]);
    for id in ids {
        println!("{}: {}", dict[&id], scores[&id]);
    }
}
