<h1 align="center">smeagol</h1>
<div align="center">A Rust library to efficiently simulate Conway's Game of Life</div>
<div align="center">
<a href="https://travis-ci.com/billyrieger/smeagol">
    <img src="https://img.shields.io/travis/com/billyrieger/smeagol.svg" alt="Build status">
</a>
<a href="https://codecov.io/gh/billyrieger/smeagol/branch/master">
    <img src="https://img.shields.io/codecov/c/github/billyrieger/smeagol.svg" alt="Coverage">
</a>
<a href="https://github.com/Aaronepower/tokei">
    <img src="https://tokei.rs/b1/github/billyrieger/smeagol" alt="Lines of code">
</a>
</div>

`smeagol` is a Rust library built to efficiently simulate large patterns in the cellular automaton
Conway's Game of Life.

## Limitations

Currently there is no garbage collection. Large patterns will eventually crash the program. This
will be fixed in the future.
