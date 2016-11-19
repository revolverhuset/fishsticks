Compile
-------

    rustup toolchain install nightly-2016-11-06
    rustup default nightly-2016-11-06
    cargo install diesel_cli
    diesel migration --database-url=dev.db run
    cargo build

Run
---

    cargo run -- --database :memory: --migrations
