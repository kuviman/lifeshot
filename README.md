# LifeShot

This is a game developed for [Ludum Dare 44](https://ldjam.com/events/ludum-dare/44).

In this game you are to survive waves of enemies. You can shoot them, but each shot costs you life. Gather food, destroy your enemies and survive as long as possible.

## Play

For web version of this game, visit https://kuviman.gitlab.io/lifeshot/.

It is also possible to build a native version from source.

## Build

To build the game from source, you'll need to install [Rust](https://rustup.rs/).

Then, just run

```shell
cargo run --release
```

To build web version, first install [`cargo-web`](https://github.com/koute/cargo-web), then run

```shell
cargo web start --release --open