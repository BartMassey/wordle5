#!/bin/sh
for l in A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
do
    n=`grep -i $l ../words-nyt-wordle.txt | egrep -v '(.).*\1' | wc -l`
    echo $n $l
done | sort -n | head -n 6
