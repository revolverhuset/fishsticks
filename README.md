[![Build Status](https://travis-ci.org/revolverhuset/fishsticks.svg?branch=master)](https://travis-ci.org/revolverhuset/fishsticks)

Compile
-------
`fishsticks` makes heavy use of procedural macros, also known as macros 1.1,
via Diesel and Serde. These are presently only available in the nightly
channel of Rust, but in contrast to compiler plugins, procedural macros are
intended to be stabilized and become available in the stable channel.

    rustup default nightly
    cargo build

If you are having problems compiling, you might have luck with running
`rustup update` and trying again.

Run
---
    cargo run -- --database :memory: --migrations
