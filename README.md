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

Single block validation benchmark is currently broken. It is benchmarking the error path of the try_extend_one. Generating large enough sets to benchmark with criterion might be imposssible due to the large number of iterations criterion does and the fact that a non 0 sample set needs to continue growing. Cloning on every iteration 
would be noisy. I might play with this in the future to fix it.
