<p align=center><img align=center src='https://user-images.githubusercontent.com/693511/77836507-0fc11300-712d-11ea-83a4-0c14b674378e.png' width=650 /></p>
<h6 align=center>Bossphorus implementation in Rust</h6>

This is a reimplementation of the [Bossphorus](https://github.com/aplbrain/bossphorus) codebase in [Rust](https://www.rust-lang.org/).


## Why?

It is bewildering how much faster this is than the Python implementation.


## Feature Parity

See [Feature Parity](docs/Features.md) for more information.


## Disk Usage

Bossphorus caches cuboids in the `uploads` folder that's created in the current
working directory.  Currently, it will cache up to 1000 cuboids in this folder.
The least recently used cuboids are removed when the cuboid limit is reached.


## Configuration

Environment variables have precedence over the `Rocket.toml` config file.


### Environment Variables

`BOSSHOST`: Sets the Boss DB host  
`BOSSTOKEN`: Token used for Boss auth


### Rocket.toml File

`bosshost`: Sets the Boss DB host  
`bosstoken`: Token used for Boss auth


### Defaults

In absence of an environment variable and value in the `Rocket.toml` file:

```
bosshost = "api.bossdb.io"
bosstoken = "public"
```


## Development

Blosc must be installed manually via a package manager to build.  SQLite is
required, but it included with MacOS by default.

For MacOS:

```shell
brew install c-blosc
```

For Debian based Linux distros:

```shell
sudo apt-get install libblosc-dev sqlite3
```

For RPM based Linux distros:

```shell
sudo yum install blosc sqlite
```


Due to use of the Rocket web server crate, the nightly Rust toolchain must be used. You can set this as your project default with:

```shell
rustup override set nightly
```


## Releases

You can build an optimized release with:

```shell
cargo build --release
```

The binary will be at `target/release/bossphorus`.
