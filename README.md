<p align=center><img align=center src='https://user-images.githubusercontent.com/693511/77836507-0fc11300-712d-11ea-83a4-0c14b674378e.png' width=650 /></p>
<h6 align=center>Bossphorus implementation in Rust</h6>

This is a reimplementation of the [Bossphorus](https://github.com/aplbrain/bossphorus) codebase in [Rust](https://www.rust-lang.org/).


## Why?

It is bewildering how much faster this is than the Python implementation.


## Feature Parity

I'm planning to add a feature-parity document once it's not embarrassing.


## Configuration

Environment variables have precedence over the `Rocket.toml` config file.

### Environment Variables

`BOSSHOST`: Sets the Boss DB host

### Rocket.toml File

`bosshost`: Sets the Boss DB host


## Development

Blosc must be installed manually via a package manager to build.

For MacOs:

```shell
brew install c-blosc
```

Due to use of the Rocket web server crate, the nightly Rust toolchain must be
used.
