# khost

Kaspa p2p node deployment automation tool for Linux.

This tool is designed to automate deployment of nodes intended for use as a part of the Kaspa public p2p network as well as private network clusters using wRPC.

## Setting up

As root:

```bash
adduser -q kaspa
sudo adduser kaspa sudo
login kaspa
```

As kaspa:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo install khost
khost
```


If you already have an existing user and rust installed, you can simply run `cargo install khost` and then `khost`.

Please note that the user needs to have root privileges to run khost.

IMPORTANT: This tool creates it's own configuration for the kaspad node, as such, any previous configurations should be disabled and removed. If kaspad was running before under the same username, rusty-kaspa data folders will be re-used.