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
    if b.bit_count() < 5:
        continue
    # Deal with anagrams.
    if b in translations:
        translations[b] += "/" + w
        continue
    translations[b] = w
print(f"{len(translations)} translations")
wsets = list(translations.keys())

def solve(i, ws, seen):
    d = len(ws)
    if d == 5:
        for w in ws:
            print(f"{translations[w]} ", end = "")
        print()
        return

    for j, w in enumerate(wsets[i:]):
        if seen & w:
            continue
        ws.append(w)
        solve(i + j + 1, ws, w | seen)
        ws.pop()

solve(0, [], 0)
