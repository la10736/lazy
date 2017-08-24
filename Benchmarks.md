# Benchmarks

On my laptop *Intel(R) Core(TM) i7-6500U CPU @ 2.50GHz* Ubuntu
Linux Kernel *4.10.0-28*

## Before use *trait objects*

| N     | No Thread  Safe    | Thread Safe          |
|------:|-------------------:|---------------------:|
| 1000  | 1,504 +/- 40       | 24,856 +/- 391       |
| 100000| 149,703 +/- 20,751 | 2,444,693 +/- 59,565 |

## Use *trait objects*

| N     | No Thread  Safe    | Thread Safe          |
|------:|-------------------:|---------------------:|
| 1000  | 3,007 +/- 36       | ????                 |
| 100000| 299,198 +/- 10,642 | ????                 |
