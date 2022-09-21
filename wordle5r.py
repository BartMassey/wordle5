#!/usr/bin/python3
# Bart Massey 2022
# Wordle5 solution in the style of Matt Parker's but with better pruning.

import sys

with open(sys.argv[1], "r") as d:
    words = d.read().strip().split('\n')
print(f"{len(words)} words")

def bits(w):
    b = 0
    for c in w:
        b |= 1 << (ord(c) - ord('a'))
    return b

translations = dict()
for w in words:
    b = bits(w)
    if b.bit_count() != 5:
        continue
    # Deal with anagrams.
    if b in translations:
        translations[b] += "/" + w
        continue
    translations[b] = w
print(f"{len(translations)} translations")
wsets = list(translations.keys())

lwords = []
for l in range(26):
    words = [w for w in wsets if (1 << l) & w]
    if words:
        lwords.append((l, words))
lwords.sort(key = lambda e: len(e[1]))

seen = 0
nlwords = []
for i in range(26):
    l, ws = lwords[i]
    nws = [ w for w in ws if (w & seen).bit_count() <= 1 ]
    if nws:
        nlwords.append((l, nws))
    seen |= 1 << l
lwords = nlwords

vowel_letters = "aeiouyw"
vowels = 0
for v in vowel_letters:
    l = ord(v) - ord('a')
    vowels |= 1 << l

def solve(i, ws, seen, skipped):
    d = len(ws)
    if d == 5:
        for w in ws:
            print(f"{translations[w]} ", end = "")
        print()
        return

    for j, es in enumerate(lwords[i:]):
        l, lws = es
        if seen & (1 << l):
            continue

        for w in lws:
            if seen & w:
                continue

            if 7 - (vowels & (w | seen)).bit_count() < 4 - d:
                continue

            ws.append(w)
            solve(i + j + 1, ws, w | seen, skipped)
            ws.pop()

        if not skipped:
            solve(i + j + 1, ws, seen, True)
        return

solve(0, [], 0, False)
