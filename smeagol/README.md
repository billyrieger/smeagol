# smeagol

## Introduction

`smeagol` is a Rust library built to efficiently simulate large patterns in the cellular automaton
Conway's Game of Life. It uses the HashLife algorithm developed by Bill Gosper to achieve tremendous
speedups for repetitive patterns. A good explanation of HashLife can be found
[here](http://www.drdobbs.com/jvm/an-algorithm-for-compressing-space-and-t/184406478).

## Usage

Add `smeagol` to your `Cargo.toml`:

```toml
[dependencies]
smeagol = "0.1"
```

Then, start simulating!

## Limitations

Currently there is no garbage collection. Large patterns will eventually crash the program. This
will be fixed in the future.
