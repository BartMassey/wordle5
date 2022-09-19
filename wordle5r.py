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

def ok_reduce(new_pvs):
    for w in wsets:
        if (w & new_pvs) == 0:
            return False
    return True

def make_reduce(make):
    def reduce(pvs, pvls):
        for i, l in reversed(list(enumerate(pvls))):
            new_pvs = pvs & ~(1 << l)
            if ok_reduce(new_pvs):
                del pvls[i]
                return new_pvs, pvls
        return pvs, pvls

    pvs, pvls = make()
    while True:
        new_pvs, new_pvls = reduce(pvs, pvls)
        if new_pvs == pvs:
            break
        pvs = new_pvs
        pvls = new_pvls
    return pvs, pvls

def pvls_str(pvls):
    result = ""
    for l in pvls:
        result += chr(ord('a') + l)
    return result

def make_pvs_lazy():
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

pvs_lazy, pvls_lazy = make_reduce(make_pvs_lazy)
print(f"pseudovowels (lazy): {pvls_str(pvls_lazy)}")
npvs_lazy = pvs_lazy.bit_count() 

def make_pvs_eager():
    lws = list([k, list(v)] for k, v in lwords.items())
    pvs = 0
    pvls = []
    while not ok_reduce(pvs):
        l = max(lws, key = lambda l : len(l[1]))
        pvls.append(l[0])
        pvs |= 1 << l[0]
        lws = [[k, list(filter(lambda w: w & pvs == 0, v))]
               for [k, v] in lws]
    return pvs, pvls

pvs_eager, pvls_eager = make_reduce(make_pvs_eager)
print(f"pseudovowels (lazy): {pvls_str(pvls_eager)}")
npvs_eager = pvs_eager.bit_count() 

pv_groups = ((pvs_eager, npvs_eager), (pvs_lazy, npvs_lazy))

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

            def vowel_prune():
                for pvs, npvs in pv_groups:
                    pvl = (w | seen) & pvs
                    npvl = pvl.bit_count()
                    if npvl + (4 - d) > npvs:
                        return True
                return False

            if vowel_prune():
                continue

            ws.append(w)
            solve(i + j + 1, ws, w | seen, skipped)
            ws.pop()

        if not skipped:
            solve(i + j + 1, ws, seen, True)
        return

solve(0, [], 0, False)
