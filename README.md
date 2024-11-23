# pumpfun fork program

## Setup

Only needed once per chain

## Deployment

Program is deployed

## Config

Call the `configure` instruction to set the config for the program +
Sets the fee amounts and allowed launch parameters.

## Prerequites

Install Rust, Solana, and AVM: https://solana.com/docs/intro/installation

Remember to install anchor v0.30.1.

## Quick Start

### Build the program

```bash
# build the program
anchor run build

# For those who use a different CARGO_TARGET_DIR location (like me I used ${userHome}/.cargo/target)
# then you'll need to move the <program-name>.so back to $PWD/target/deploy/<program-name.so>.

# E.g:
ln -s $HOME/.cargo/target/sbf-solana-solana/release/pumpfun.so $PWD/target/deploy/pumpfun.so
```

### Run tests

you can run the tests without having to start a local network:

```bash
anchor test
```

### Start a local network and run tests

Run a local Solana validator network:

```bash
solana config set -ul    # For localhost

solana config set -k ./testKeypair.json # use the test keypair for simplicity

# start a localhost testnet completely fresh
# --bpf-program is for init programs at genesis. We need metadata program.
# another way is to use --clone using --url as reference. Ref: https://www.anchor-lang.com/docs/manifest#test-validator
solana-test-validator -r --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s spl-programs/metadata.so
```

Deploy the program:

```bash
anchor deploy
```

Run some tests:

```bash
# run all tests
anchor run test

# run a single test (e.g. a test with "correctly configured" as name)
anchor run test -- "'correctly configured'"
```
