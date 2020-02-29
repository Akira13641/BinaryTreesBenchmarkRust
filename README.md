[![Build Status](https://travis-ci.com/akira13641/binarytreesbenchmark.svg?branch=master)](https://travis-ci.com/akira13641/binarytreesbenchmarkrust)

A port of a combination of the D and C++ ports of my [Free Pascal](https://benchmarksgame-team.pages.debian.net/benchmarksgame/program/binarytrees-fpascal-7.html) "Binary Trees" benchmark implementation, once again written just out of curiosity to see how they all compare.

Recommended command line for building and running this:

```
cargo build --release
cd ./target/release
time ./binarytrees_benchmark 21
```
