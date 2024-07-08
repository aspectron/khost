# khost

Kaspa p2p public node management tool.

Work in progress - this is not suitable for production use yet.

As root:

```bash
adduser -q kaspa
login kaspa
```

As kaspa:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo install khost
sudo khost
```

If you already have an existing user and rust installed, you can simply run `cargo install khost` and `sudo khost`.

Please note that the user needs to have root privileges to run khost.

IMPORTANT: This tool creates it's own configuration for the kaspad node, as such, any previous configurations should be disabled. If kaspad was running before under the same username, rusty-kaspa data folders will be re-used.