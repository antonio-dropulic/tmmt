# TMMT - Teenage mutant miner turtles

This project is a coding challenge. The challenge said something about turtles.
The name is my attempt at a joke :D.

## Challenge

The first 100 numbers of the mine are always secure, but after that, the next number is only safe if it is the sum of 2 numbers in the previous 100.

Test data given in the `resources/challenge_input.txt`. Unclear what the result should be.

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

This test is implemented and passing in `tests::example`.


## Todo:

- incomplete test suite
- perf tests
- memory profiling
- clean up docs and naming
- Semantic typing for Block and BlockSum

- streaming api
- reporting empty runs of the mine