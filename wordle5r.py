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

lwords = dict()
for l in range(26):
    words = [w for w in wsets if (1 << l) & w]
    lwords[l] = words

letters = [l for _, l in sorted([(len(lwords[l]), l) for l in range(26)])]

def solve(i, ws, seen, skipped):
    d = len(ws)
    if d == 5:
        if seen.bit_count() < 25:
            return
        for w in ws:
            print(f"{translations[w]} ", end = "")
        print()
        return

    for j, l in enumerate(letters[i:]):
        if seen & (1 << l):
            continue

        for w in lwords[l]:
            if seen & w:
                continue

            ws.append(w)
            solve(i + j + 1, ws, w | seen, skipped)
            ws.pop()

        if not skipped:
            solve(i + j + 1, ws, seen, True)
        return

solve(0, [], 0, False)
