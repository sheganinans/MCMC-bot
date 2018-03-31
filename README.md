```bash
curl https://nixos.org/nix/install | sh
# follow nix post-install instructions

git clone https://github.com/sheganinans/MCMC-bot.git
cd MCMC-bot/
nix-shell
DISCORD_TOKEN="..." cargo run --release
```
