Compile
-------
`fishsticks` makes heavy use of procedural macros, also known as macros 1.1,
via Diesel and Serde. These are presently only available in the nightly
channel of Rust, but in contrast to compiler plugins, procedural macros are
intended to be stabilized and become available in the stable channel.

    rustup toolchain install nightly-2016-11-06
    rustup default nightly-2016-11-06
    cargo build

Run
---
    cargo run -- --database :memory: --migrations
