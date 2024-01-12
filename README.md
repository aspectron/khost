# khost

Bootstrap scripts for Linux Kaspa p2p Nodes

As root:

```bash
adduser kaspa
chmod g+r /home/kaspa
```

As kaspa:

```bash
auto apt install -y git
git clone https://github.com/aspectron/khost
cd khost/bootstrap
./init
```
