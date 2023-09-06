# TMMT - Teenage mutant miner turtles

This project is a coding challenge. The challenge said something about turtles.

## Challenge

The first 100 numbers of the mine are always secure, but after that, the next number is only safe if it is the sum of 2 numbers in the previous 100.

Test data given in the `resources/challenge_input.txt`. Unclear what the result should be.
Some inputs in `resources/challenge_input.txt` are larger than `u64::MAX` but all are smaller
than `u128::MAX / 2`.

Test of the input data: https://github.com/antonio-dropulic/tmmt/blob/c70e887fd7b191b92a9f208749ed48108b748067/src/lib.rs#L327-L344

Smaller scale test was also provided for a window of 5 instead of 100.

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

This test is implemented and passing in [`tests::example`](https://github.com/antonio-dropulic/tmmt/blob/ad94a468d3d265b9bf0ada6ac4c6c767fe7df800/src/lib.rs#L263-L286).

## Notes:

Two competing implementations are given. One using the hash map and one using the two pointer approach. Initially I thought of initialization as negligible - validation window is limited, number of incoming blocks isn't - and the idea was to have `try_extend_one` to be as fast as possible.
That is what hash map implementation was meant to do. Having difficulty creating a criterion test for `try_extend_one` I realized I have misjudged the problem.

The key idea I missed when analyzing the problem was that the sequence of values that can be given for validation is limited. If we assume the initial values are non zero (Idea not enforced in the code, but still reasonable given uniform datasets). Then the
number of iterations over `try_extend_one` is necessarily bounded. Imagine we start with a validation window size N with all ones: `[1; N]`. The smallest value we can then validate is 2. and we can do that N - 1 times before we have to double the smallest value.
That means the iteration count is bounded by $N * Block::BITS$.

This suggests `TwoPointerMine` is better. Current tests reflect that as `create_and_extend _many` runs significantly faster for said implementation.
