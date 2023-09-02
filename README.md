# TMMT - Teenage mutant miner turtles

This project is a coding challenge. The challenge said something about turtles.

## Challenge

The first 100 numbers of the mine are always secure, but after that, the next number is only safe if it is the sum of 2 numbers in the previous 100.

Test data given in the `resources/challenge_input.txt`. Unclear what the result should be.
Some inputs in `resources/challenge_input.txt` are larger than `u64::MAX` but all are smaller
than `u128::MAX / 2`.

Test of the input data: https://github.com/antonio-dropulic/tmmt/blob/ad94a468d3d265b9bf0ada6ac4c6c767fe7df800/src/lib.rs#L288-L306

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

I've played around with two pointer implementation of the validation algorithm.
Obviously it made initialization much faster. $O(log(N))$ instead $O(N^2)$ where N is validation window size.
But the validation / insertion was much slower - more than 5x times with the validation window size 100 using u128 as block.
It seemed to scale linearly. Somehow the current implementation for small validation window sizes **(5, 50, 100)** seems to perform in
$O(1)$. On my machine running [`Mine::try_extend_one`](https://github.com/antonio-dropulic/tmmt/blob/15bf1e971d5cb9b0da844221c147bd49304d016c/src/lib.rs#L83-L110) (for valid inputs) on the mentioned window sizes I measure 18 ns avg for all of them.
