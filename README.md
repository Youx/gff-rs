
# gff-rs &emsp; [![Rust](https://github.com/Youx/gff-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/Youx/gff-rs/actions/workflows/rust.yml)

Library for the GFF file format, used in some BioWare games

This library is composed of two parts:

## gff

`gff` provides an intermediary data representation, common types,
a parser and a packer for the GFF format.

This allows you to open a file (like the `.bic` file provided for tests,
one of my online NWN character sheet).

These files can then be decoded to intermediary representation, modified
and repacked.

## gff-derive

`gff-derive` provides procedural macros to automatically derive traits
that allow you to transform any (compatible) Rust `struct` from/into
the intermediary GFF representation (can then be packed).

Work is in progress to provide direct GFF <-> `struct` support.

# TODO

- implement direct packing/parsing
- support encodings for more games
