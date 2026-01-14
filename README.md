# Splitl

This utility takes an input stream and splits it into lines, effectively susbtituting a choosen character with a newline `\n`.

# Motivation
I did not know `tr` existed and I hate `sed` syntax.

```bash
tr ',' '\n' < worldcitiespop.csv
```

Anyway, `splitl` is limited by single character substitutions but it's around 8x faster because it employs SIMD. And it's nearly as fast as cat. And it's written in Rust.

The following benchmark uses [world cities population dataset](https://github.com/petewarden/dstkdata/blob/master/worldcitiespop.csv) (12).
It's done on a Macbook Pro with Apple M3 Pro; I am using gtr which is the gnu version of the `tr` tool from the gnu utilities, basically the version you find on linux and not the one find by default on mac which is different.
```bash
hyperfine "gtr ',' '\n' <  worldcitiespop.csv"
Benchmark 1: gtr ',' '\n' <  worldcitiespop.csv
  Time (mean ± σ):      87.7 ms ±   2.1 ms    [User: 47.3 ms, System: 39.6 ms]
  Range (min … max):    86.3 ms …  97.7 ms    32 runs
 

hyperfine "./target/release/splitl -d, worldcitiespop.csv" 
Benchmark 1: ./target/release/splitl -d, worldcitiespop.csv
  Time (mean ± σ):      10.6 ms ±   0.3 ms    [User: 2.5 ms, System: 7.8 ms]
  Range (min … max):    10.1 ms …  11.3 ms    216 runs

hyperfine "cat worldcitiespop.csv" 
Benchmark 1: cat worldcitiespop.csv
  Time (mean ± σ):       9.5 ms ±   0.3 ms    [User: 0.8 ms, System: 8.4 ms]
  Range (min … max):     9.0 ms …  10.6 ms    222 runs 
```

When using stdin, due to `cat` timing, the speedup is more around 6x:

```bash
hyperfine "cat worldcitiespop.csv | gtr ',' '\n"
Benchmark 1: cat worldcitiespop.csv | gtr ',' '\n'
  Time (mean ± σ):     118.6 ms ±   2.7 ms    [User: 60.2 ms, System: 96.4 ms]
  Range (min … max):   113.0 ms … 123.2 ms    23 runs

hyperfine "cat worldcitiespop.csv | ./target/release/splitl -d,"
Benchmark 1: cat worldcitiespop.csv | ./target/release/splitl -d,
  Time (mean ± σ):      20.7 ms ±   0.5 ms    [User: 5.7 ms, System: 29.3 ms]
  Range (min … max):    19.8 ms …  23.5 ms    127 runs
```
