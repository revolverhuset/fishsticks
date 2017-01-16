[![Build Status](https://travis-ci.org/revolverhuset/fishsticks.svg?branch=master)](https://travis-ci.org/revolverhuset/fishsticks)

Compile
=======
`fishsticks` makes heavy use of procedural macros, also known as macros 1.1.
These are presently only available in the beta and nightly channels of Rust.
Procedural macros are expected to become available in stable Rust with version
1.15, to be released February 3.

    rustup override set nightly
    cargo build

If you are having problems compiling, you might have luck with running
`rustup update` and trying again.

Run
===
    cargo run -- --database :memory: --migrations
