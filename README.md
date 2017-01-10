[![Build Status](https://travis-ci.org/revolverhuset/fishsticks.svg?branch=master)](https://travis-ci.org/revolverhuset/fishsticks)

Compile
=======
`fishsticks` makes heavy use of procedural macros, also known as macros 1.1.
These are presently only available in the beta and nightly channels of Rust.
Procedural macros are expected to be available in stable Rust version 1.15, to
be released February 3.

    rustup override set nightly
    cargo build

If you are having problems compiling, you might have luck with running
`rustup update` and trying again.

OpenSSL on Mac OS X
-------------------
If you are having problems relating to OpenSSL when compiling on OS X, replace
the `hyper`-line in `Cargo.toml` with something like:

    hyper = { version = "^0.9.12", default_features = false, features = ["security-framework"] }

Don't commit this change, because it breaks the build outside of OS X.

`fishsticks` uses `hyper` for performing HTTP and HTTPS requests. Sharebill is
behind HTTPS, so this is a required feature. For TLS, `hyper` uses OpenSSL by
default. This requires a native OpenSSL installation to be provided. This is a
no-brainer on Debian (`apt install libssl-dev`), but silly and confusing and
potentially dangerous on OS X.

It is known that the [openssl dependency doesn't work](https://github.com/hyperium/hyper/issues/709).

It is currently unfortunately [impossible to configure cargo](https://github.com/rust-lang/cargo/issues/3195)
to automatically use `security-framework` for OS X while automatically using
OpenSSL otherwise.

In the future, [`hyper` might depend on rustls](https://github.com/hyperium/hyper/issues/956)
instead of OpenSSL. This would make the entire problem go away.

Run
===
    cargo run -- --database :memory: --migrations
