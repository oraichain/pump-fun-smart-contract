## Basic smart contract for pump.fun

You can check frontend and backend repo as well.

You can contact me if you want a better product.

New features on an updated version

- All handled in smart contracts
  Pumpfun uses backend client code to fetch buy transaction and create raydium pool.
  I've handled all that parts on smart contract to enhance the security and availability.

- Added some launch phases
  There's some phases to launch a token like `Presale`.
  If the users want to snipe a token, they can bid for the token in `Presale` phase before `Launch`.

- Raydium/Meteora
  Token launchers can migrate their tokens to Raydium or Migrate as their wish after the curve is completed.

Telegram: https://t.me/microgift28

Discord: https://discord.com/users/1074514238325927956

## Prerequites

Install Rust, Solana, and AVM: https://solana.com/docs/intro/installation

Remember to install anchor v0.29.0 for stability.

## Quick Start

### Update Program ID for localnet

Run: `anchor keys sync`

### Build the program

```bash
# build the program
anchor build

# For those who use a different CARGO_TARGET_DIR location (like me I used ${userHome}/.cargo/target)
# then you'll need to move the <program-name>.so back to $PWD/target/deploy/<program-name.so>.

# E.g:
cp $HOME/.cargo/target/sbf-solana-solana/release/pump.so $PWD/target/deploy/pump.so
```

### Run tests

Run a local Solana validator network:

```bash
solana config set -ul    # For localhost

solana config set -k ./id.json # use the test keypair for simplicity

# start a localhost testnet completely fresh
solana-test-validator -r
```

Run some tests:

```bash
# run all tests
anchor run test

# run a single test (e.g. a test with "Initialize" as name)
anchor run test -- "Initialize"
```

Deploy the program:

```bash
# replace the wallet with your wallet.
anchor deploy --provider.wallet ./id.json
```
