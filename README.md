# cargo-crate-src
cargo-crate-src is a cargo subcommand to clone repositories from dependencies in Cargo.toml, so you can `git log`, `git grep` and read code in your editor there!

## Usage
```
$ export CRATESRC_CLONE_ROOT="/path/to/clone/root/"
$ cd /path/to/crate
$ path/to/cargo-crate-src/target/release/cargo-crate-src
```

## Contributing
1. Fork
2. Create a feature branch
3. Commit your changes
4. Rebase your local changes against the master branch
5. Run test suite with the `cargo test` command and confirm that it passes
6. Run `cargo fmt` and pass `cargo clippy`
7. Create new Pull Request

## License
[MIT license](LICENSE)
