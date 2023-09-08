# TMMT - Teenage mutant miner turtles

This project is a coding challenge. The challenge said something about turtles.

## Challenge

The first 100 numbers of the mine are always secure, but after that, the next number is only safe if it is the sum of 2 numbers in the previous 100.

Test data given in the `resources/challenge_input.txt`.
Inspection revealed all some values in `resources/challenge_input.txt` are larger than `u64::MAX`, while all values are smaller
than `u128::MAX / 2`.

Smaller scale test was also provided for a window of 5 instead of 100. Expected failing value is 127.

```
35
20
15
25
47
40
62
55
65
95
102
117
150
182
127
219
299
277
309
576
```

## Solution discussion

Naming convention:

- $B$ - short for block, type of the numerical values in the mine
- $I$ - validation sequence size, size of initial data.
- $N$ - number of iterations for valid blocks after initialization

##

After the initial read through motivation for the api and implementation was driven by:
- $I$ is fixed
- $N >> I$

My focus was to create a fast method for validating entries after initialization. I call these methods `Mine::try_extend one` and `Mine::new`. Two implementations came to mind.
1) Solve the two sum problem using two pointer technique over a sorted validation sequence - `TwoPtrMine`
2) Solve the two sum problem by precomputing all possible sums and storing them in a HashSet - `HashMine`

Both solutions have $O(I)$ running time for `Mine::try_extend one`. Running time for `TwoPtrMine::new` is $O(Ilog_2I)$ and $O(I)$ memory. `HashMine::new` is $O(I^2)$ running time and memory.

This should signal the 1st solution is superior but I wanted to test how `Mine::try_extend_one` performed in both implementations[^1]. Since use cases are unknown, maybe sacrificing initialization performance is worth for possible marginal gains in performance of validating new entries.

##

I've created both solutions and used criterion to test the performance. Expectedly `TwoPtrMine::new` performed better. But that was not what I was interested in testing. Testing `Mine::try_extend_one` was unexpectedly hard. I got results that didn't depend on $I$. This was a red flag, but it took me some time to realize I was only testing the error path.

 A walk with my duck got me to realize I've misjudged the problem. The key idea I initially missed was that $N$ is bounded. To see why imagine the initialization set to be as small as
possible[^2]: $[1;I]$. Then the smallest valid value we can give to `Mine::try_extend_one` is **2**, and crucially we can only do that $I-1$ times.
Generally speaking every I entries the smallest valid entry value must be doubled[^3].
By continuing this logic we can see $N < I \cdot B::Bits$. In the case of $B = u128$ and $I=100$ that is merely $12 800$ iterations.

This informed me that trying to test just the running time of `Mine::try_extend_one` is probably not a good idea. I've moved on
to testing initialization and iteration over a large set of values. This way the 1st solution proved vastly superior. {TODO: link criterion}

##

The challenge was deceptively simple, but turned out to be somewhat tricky. I'm keeping both solutions and the explanation because I found the journey fun and hopefully it can be informative to someone else.

[^1]: This was probably a futile attempt. *I* reads and writes to a hash map should be expensive.
[^2]: having [0,0, 1, ..] would be trivial.
[^3]: this is not precise but the bound remains true. I+1th value must be doubled.
