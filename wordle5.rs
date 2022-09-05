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

fn make_letter_ids(ids: &[u32]) -> Vec<(u32, Vec<u32>)> {
    let mut letter_ids = Vec::new();
    for c in 0..26 {
        let cs: Vec<u32> = ids
            .iter()
            .filter(|&&id| id & (1 << c) != 0)
            .copied()
            .collect();
        letter_ids.push((c, cs));
    }
    letter_ids.sort_unstable_by_key(|(_, cs)| cs.len());
    letter_ids
}

#[test]
fn test_make_letter_ids() {
    let letter_ids = make_letter_ids(&[0b11111]);
    for (c, v) in letter_ids {
        if c < 5 {
            assert_eq!(v, [0b11111]);
        } else {
            assert!(v.is_empty());
        }
    }
}

fn solvify(
    letter_ids: &[(u32, Vec<u32>)],
    cur: &mut [u32; 5],
    solns: &mut Vec<[u32; 5]>,
    mut posn: usize,
    count: usize,
    seen: u32,
    skipped: bool,
) {
    if count == 5 {
        solns.push(*cur);
        return;
    }

    loop {
        if posn >= 26 {
            return;
        }
        let c = letter_ids[posn].0;
        if seen & (1 << c) == 0 {
            break;
        }
        posn += 1;
    }

    for &id in &letter_ids[posn].1 {
        if seen & id != 0 {
            continue;
        }
        cur[count] = id;
        solvify(letter_ids, cur, solns, posn + 1, count + 1, seen | id, skipped);
    }
    if !skipped {
        solvify(letter_ids, cur, solns, posn + 1, count, seen, true);
    }
}

fn solve(letter_ids: &[(u32, Vec<u32>)]) -> Vec<[u32; 5]> {
    let mut partial = [0; 5];
    let mut solns = Vec::new();
    solvify(letter_ids, &mut partial, &mut solns, 0, 0, 0, false);
    solns
}

fn main() {
    let dict = assemble_dicts();
    let ids: Vec<u32> = dict.keys().copied().collect();
    let letter_ids = make_letter_ids(&ids);

    for soln in solve(&letter_ids).into_iter() {
        for id in soln {
            print!("{} ", dict[&id]);
        }
        println!();
    }
}
