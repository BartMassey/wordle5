# Solving Wordle5 Blazingly Fast With State-Space Search In Rust
Bart Massey 2022-09-10

I found an interesting problem online about six weeks
ago. YouTuber Matt Parker made a video for his excellent
channel Stand-Up Maths which proposed and solved a problem
related to Wordle. I don't think Parker gave the problem a
name, so I'm going to call it Wordle5.

I've linked Parker's video in the description:
<https://youtu.be/_-AfhLQfb6w>.

Parker wrote a program that solves this problem in a
month. Another YouTuber wrote a program that solves it in
100 milliseconds. I wrote a program that solves it in under
10 milliseconds. This video is that story.

For our purposes, it doesn't much matter if you've never
heard of Wordle.  The Wordle game page is linked in the
description if you want to check it out:
<https://www.nytimes.com/games/wordle/index.html>.

The "Wordle5 problem" is to find a set of five "legal"
five-letter words that together contain 25 of the 26 letters
of the English alphabet. The Wordle part of the problem is
that the legal words are those found in the dictionaries of
the Wordle game.

## Understanding The Problem

The first step is to find out what the legal guess words for
Wordle are: the "dictionary" for the puzzle. Fortunately
Parker has supplied us with some official New York Times
Wordle dictionary; it looks the same as other sources on the
Internet. It doesn't really matter, though: our job is to
work with any dictionary we are given, and this one will
work for comparing.

This dictionary has 12,945 words, most of them pretty
wacky. Here's the first five words

    AAHED
    AALII
    AARGH
    AARTI
    ABACA

Most of these aren't really words, but that's OK.

Given that dictionary, here's the subjectively "best"
solution

    FJORD
    VIBEX
    WALTZ
    GUCKS
    NYMPH

"Best" here means the most "real" words: VIBEX is pretty
garbage, and GUCKS is questionable. You might prefer this

    FJORD
    VIBEX
    WALTZ
    GYMPS
    CHUNK

but honestly GYMPS seems worse than GUCKS to me. Both of
these solutions leave out Q. There are eight more solutions,
but they all use both of these words

    WAQFS
    VOZHD
   
so… no.

Finding 25 different letters in 25 positions means that
there can be no duplicate letters within a word.  I used a
UNIX command to filter out the words with duplicate letters
and count the rest.

    egrep -v '(.).*\1' words-nyt-wordle.txt | wc -l

Turns out we're down to 8,310 words.

## Understanding Parker's Solver

Matt Parker proposed this problem, and Matt Parker solved
it. His initial solution was written in Python, was very
simple, and ran for more than 31 days. 31 days.

The dumb way to solve Wordle5 is to pick every combination
of 5 words from our 8,310-word filtered dictionary and see
if it's a solution. Some fancy math will show that the
number of such combinations, written

    choose5.png

and pronounced "8310 choose 5", is more than 10^17 (1 with
17 zeros). This way madness lies. The fastest supercomputer
available today would likely take a week to check all the
possibilities — it would cost at least tens of thousands of
dollars.

Parker's plan was to take some advantage of the fact that a
solution can be built up piece by piece. If we try to find
*pairs* of compatible words, there's "only" about 69 million
pairs to check. Presumably most of these won't go together,
so we will be in a better place to continue. I've replicated
Parker's solution in cleaner Python, and it shows about 2.4
million word pairs (found in a little more than half a
second), which again is manageable for a computer.

Parker chose to proceed by trying to extend each pair of
legal word pairs to get a four-letter group. That's about
5.7 trillion groups to consider, so we won't be trying
that. For each of these four-letter groups, Parker
tried to extend the group to a five-letter
solution. Frankly, I'm amazed it only took a month on
Parker's box in Python.

Instead, my cleaned-up version of Parker's Python solution
tries to extend a solution *one word at a time.* This is
just objectively better: since most extensions won't
succeed, we'll get a lot of "pruning".

There's apparently about 1500 ways to extend an "average"
word to a two-word solution, but most of those extensions
won't extend to three words, much less four. The classic way
to write this approach is using *recursion*, like this:

    To show all solutions at depth d, given a partial solution P:
         If d is five, P is a solution. Display it.
         Otherwise, for each word w in the dictionary
             If w is compatible with P
                 Show all solutions at depth d + 1 with P + w

    Show all solutions at depth 0 with partial solution P empty

There's some fancy ways to speed this up, but that's the
basic idea.

My Python code, like Parker's, takes a long time to
run. Unlike Parker's, it's an hour instead of a month: a big
savings. This approach finds four-word partial solutions
relatively quickly: they are not rare. But only 10 of them
extend to five words. For the record, it's these 10:

    BEMIX CLUNK GRYPT VOZHD WAQFS 
    BLING JUMPY TRECK VOZHD WAQFS 
    BLUNK CIMEX GRYPT VOZHD WAQFS 
    BRICK GLENT JUMPY VOZHD WAQFS 
    BRUNG CYLIX/XYLIC KEMPT VOZHD WAQFS 
    CHUNK FJORD GYMPS VIBEX WALTZ 
    CLIPT JUMBY KRENG VOZHD WAQFS 
    FJORD GUCKS NYMPH VIBEX WALTZ 
    GLENT JUMBY PRICK VOZHD WAQFS 
    JUMBY PLING TRECK VOZHD WAQFS 

These solutions are in alphabetical order, since that's the
order we're looking at words in. "CYLIX/XYLIC" is because
these so-called words are anagrams: you can make one from
the other by rearranging the letters.

## Looking At The Internet's Solvers

When I saw Parker's Wordle5 on YouTube, I thought to myself
"I can make this run *really* fast. But so can half the
serious programmers on the Internet. Let's see what they
do."

Sure enough, people used a bunch of tricks and sped things
up. In about a month, the runtime of the solvers were down
below one second.  YouTuber Fred Overflow broke 100
milliseconds around September 1st. His video is linked
below: <https://youtu.be/Y37WiO55bxs>

Fred Overflow's 100 milliseconds is seven orders of
magnitude faster than Parker. 10 million times faster. Part
of that is using Golang, a compiled language that runs up to
100 times faster than Python. Still, a factor of at least
100,000 from better search. Nice.

That's where it would have ended for me, except a Redditor
asked for help speeding up their Rustlang version of Fred
Overflow's 100 millisecond program. So I did. But looking at Fred
Overflow's time, I said to myself "OK. I have a PhD in State
Space search. I'm supposed to know how to do this stuff. I
wonder if I can do it in *one* millisecond?"

Spoiler alert: I can't. But I'm around 5ms. The best
credible solution I've seen reported so far is 25ms. The
rest of this video is about how I got this result.

## Rewriting It In Rust

Let's start by taking my "pseudo-Parker" solution and
rewriting it in Rust. Rust is a great programming language
that I am quite happy with. It compiles to very fast code
and is safe against common mistakes you might make in other
programming languages.

You will find my first Python solution together with my
Rust rewrite in the branch `retro2` in the `wordle5` repo.
The Python runs in about 67 minutes. The Rust takes about 20
seconds, so it's about 180 times faster than the Python.

## Switching Search Spaces

I first sat down to write my Rust version a couple of
weeks ago. At that point I already knew to skip the
algorithm I've described so far — it's too slow. The trick
to these state-space search things is to figure out the
right space to search in.

Think about the six letters that appear in the least
words. For us that's

    92 Q
    192 J
    215 X
    239 Z
    462 V
    646 F

which I found with a UNIX command

    for l in A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
    do
        n=`grep -i $l words-nyt-wordle.txt | egrep -v '(.).*\1' | wc -l`
        echo $n $l
    done | sort -n | head -n 6

We know we have to have words that together contain five of
these six letters in any final solution. So let's try our
words in that order: first try all words that contain a Q,
then try to find compatible words that contain an X, etc.

There are some exceptions that will happen. Maybe the first
word is SQUIZ, which contains both a Q and a Z, or maybe we
decide to start by skipping Q and using the other five
letters.  In any case, by the time we've dealt with these
hard-to-use letters we'll have a lot of letters and words
used up. That means we'll have only one or two words to
place, and few choices for what they are.

Our algorithm now looks something like this

    Make a list L of letters in increasing frequency order

    To show all solutions at depth d, given a position i in L
      and a partial solution P:
         If d is five, P is a solution. Display it.
         Find the letter l at the first position j >= i
           in L that is ok with the partial solution
         For each word w in the dictionary containing l
             If w is compatible with P
                 Show all solutions at depth d + 1 with j + 1, P + w
         Show all solutions at depth d with position j + 1, P (skip l)

    Show all solutions at depth 0 with empty partial solution and i at 0

I've programmed this solution in both Rust and Python. The
Python runs in about one second: more than 3600 times faster
than it ran before, 20 times faster than the old Rust
version. The Rust runs in about 15 milliseconds, about 70
times faster than the Python.

This is the power of good search: finding the right search
space is everything. No amount of speeding up the old bad
search implementation would have got us here.

## Remove "Impossible" Words From The Search



## Stop When Out Of Vowels

Some playing with the dictionary shows that all but one of
our 8,310 words contains an A, E, I, O, U or Y. That one is

    CRWTH

which uses W as a vowel. This is annoying, but such is life.
Anyway, every word contains an A, E, I, O, U, Y or W. So
there's that.
