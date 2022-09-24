#!/usr/bin/python3
# Bart Massey 2022
# Wordle5 solution in the style of Matt Parker's but with better pruning.

import multiprocessing
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

def to_chars(letters):
    return ''.join(chr(ord('a') + l) for l in letters)

vowels = 0
vowel_letters = []
remaining = list(wsets)
for l, _ in reversed(lwords):
    vowel_letters.append(l)
    vowels |= 1 << l
    remaining = [w for w in remaining if (w & vowels) == 0]
    if not remaining:
        break

gvowels = 0
gvowel_letters = []
glwords = [[l, list(ws)] for l, ws in lwords]
while glwords:
    l, _ = max(glwords, key = lambda g : len(g[1]))
    gvowel_letters.append(l)
    gvowels |= 1 << l
    for g in glwords:
        g[1] = list(filter(lambda w: (w & gvowels) == 0, g[1]))
    glwords = list(filter(lambda g: g[1], glwords))

def prune(vowel_letters, vowels):
    cvowels = vowels
    candidates = reversed(vowel_letters)
    cletters = [next(candidates)]
    for l in candidates:
        cvowels &= ~(1 << l)
        for w in wsets:
            if (w & cvowels) == 0:
                cvowels |= 1 << l
                cletters.append(l)
                break
    return list(reversed(cletters)), cvowels

vowel_letters, vowels = prune(vowel_letters, vowels)
nvowels = vowels.bit_count()
print(f"pseudovowels: {to_chars(vowel_letters)}")

gvowel_letters, gvowels = prune(gvowel_letters, gvowels)
ngvowels = gvowels.bit_count()
print(f"greedy pseudovowels: {to_chars(gvowel_letters)}")

seen = 0
nlwords = []
for i in range(26):
    l, ws = lwords[i]
    nws = [ w for w in ws if (w & seen).bit_count() <= 1 ]
    if nws:
        nlwords.append((l, nws))
    seen |= 1 << l
lwords = nlwords

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

            if nvowels - (vowels & (w | seen)).bit_count() < 4 - d:
                continue

            if ngvowels - (gvowels & (w | seen)).bit_count() < 4 - d:
                continue

            ws.append(w)
            solve(i + j + 1, ws, w | seen, skipped)
            ws.pop()

        if not skipped:
            solve(i + j + 1, ws, seen, True)
        return

def solve1(w):
    solve(1, [w], w, False)

def solve2(w):
    solve(2, [w], w, True)

pool = multiprocessing.Pool()
pool.map(solve1, lwords[0][1])
pool.map(solve2, lwords[1][1])
