## `kHOST`

[<img alt="github" src="https://img.shields.io/badge/github-aspectron/khost-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/aspectron/khost)
<img alt="license" src="https://img.shields.io/crates/l/kHOST.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">

Kaspa p2p node deployment automation tool for Linux.

kHOST was created to automate deployment of Kaspa nodes intended for use as a part of the Kaspa public RPC network as well as private network high-availability clusters.  kHOST deploys Rusty-Kaspa nodes from sources, configures them to run as a `systemd` service as well as configures NGINX to act as a reverse proxy for the RPC.  This tool exists to simplify and automate Kaspa node deployment as well as to standardize related system configuration.

## Deploying

As `root`:

```bash
sudo -s
apt install -y curl build-essential pkg-config libssl-dev
adduser -q kaspa
adduser kaspa sudo
login kaspa
```

As `kaspa` user:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
cargo install khost
khost
```

If you already have an existing user and rust installed, you can simply run `cargo install khost` followed by `khost`.

Please note that the user needs to have root (sudo) privileges to run khost.

IMPORTANT: This tool creates it's own configuration for the kaspad node, as such, any previous configurations should be disabled and removed. If kaspad was running before under the same username, the `~/.rusty-kaspa` data folders containing databases will be re-used.
