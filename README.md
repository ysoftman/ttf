# ttf

Terminal Tool Finder — a CLI tool that fuzzy-searches terminal commands in `tools.json`.

## install and usage

```bash
# install (crates.io)
cargo uninstall terminal-tool-finder; cargo install terminal-tool-finder

# reinstall (if already installed)
cargo install terminal-tool-finder --force

# install (build from local source)
cargo build --release
# binary: target/release/ttf

# help
ttf -h

# fuzzy search
ttf <query>

# interactive fuzzy finder popup (requires fzf)
# opens an fzf popup with all tools; type to filter, Enter to pick
ttf

# list all tools
ttf -l, --list

# specify tools.json path (default: exe directory or ./tools.json)
ttf -d <path>

# max number of results (default: 20)
ttf -n <n>

# disable colored output (default: color on, command names/tags colored)
ttf --nocolor
```

## build and deploy

```bash
# local test
cargo test

# run without building
cargo run -- <query>

# cargo login
# create a token at <https://crates.io/me>
# the token is saved to ~/.cargo/credentials.toml after login
cargo login

# bump version in cargo.toml -> git commit -> deploy with cargo
# --allow-dirty: allow publishing with uncommitted local changes
cargo publish
```
