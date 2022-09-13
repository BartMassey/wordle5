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

def make_pseudovowels_lazy():
    def make():
        ls = reversed(letters)
        ws = list(wsets)
        pvs = 0
        pvls = []
        for l in ls:
            pvls.append(l)
            x = 1 << l
            pvs |= x
            ws = [w for w in ws if (w & x) == 0]
            if not ws:
                return pvs, pvls
        assert False, "ran out of letters for pseudovowels"

    def reduce(pvs, pvls):
        def ok_reduce(new_pvs):
            for w in wsets:
                if (w & new_pvs) == 0:
                    return False
            return True

        for i, l in reversed(list(enumerate(pvls))):
            new_pvs = pvs & ~(1 << l)
            if ok_reduce(new_pvs):
                del pvls[i]
                return new_pvs
        return pvs

    pvs, pvls = make()
    while True:
        new_pvs = reduce(pvs, pvls)
        if new_pvs == pvs:
            break
        pvs = new_pvs
    print("pseudovowels:", ''.join([chr(l + ord('a')) for l in pvls]))
    return pvs
pvs_lazy = make_pseudovowels_lazy()
npvs_lazy = pvs_lazy.bit_count() 

def solve(i, ws, seen, skipped):
    d = len(ws)
    if d == 5:
        for w in ws:
            print(f"{translations[w]} ", end = "")
        print()
        return

    for j, l in enumerate(letters[i:]):
        if seen & (1 << l):
            continue

        for w in lwords[l]:
            if w & seen:
                continue

            pvl = (w | seen) & pvs_lazy
            npvl = pvl.bit_count()
            if npvl + (4 - d) > npvs_lazy:
                continue

            ws.append(w)
            solve(i + j + 1, ws, w | seen, skipped)
            ws.pop()

        if not skipped:
            solve(i + j + 1, ws, seen, True)
        return

solve(0, [], 0, False)
