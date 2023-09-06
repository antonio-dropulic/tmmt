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

I've played around with two pointer implementation of the validation algorithm. It made initialization much faster. $O(log(N))$ instead $O(N^2)$ where N is validation window size. But validation time was still slower. Both validation implementations are bounded by $O(N)$.

My initial idea driving the implementation was that validating elements and extending the mine is more important than initialization.
If assume that block values in the mine are non zero, or at least that the initialization sequence has less than two zeroes. Then we come
to an iteration count bound on every mine: $VALIDATION_WINDOW_SIZE * log(Block::SIZE) - 1$. Imagine we start with a validation window of all ones. Next validation window that has the smallest elements possible is a validation window [2,2, ... , 3]. Next one doubles, and so forth.

This suggests that my initial idea was flawed. I now want to make tests that test the duration of creation and maximum number of iterations. (Current tests seem decent enough). This may change my perception of how the two pointer implementation performs.
