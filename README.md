
# Event Ticketing Platform for MPL Core Leveraging the Appdata Plugin

This Anchor program leverages the [MPL Core](https://github.com/metaplex-foundation/mpl-core) AppData Plugin to create a ticketing solution that could be used to generate tickets as digital assets and verified by an external source of trust other than the issuer, like for example a venue manager.

It is the full implementation of the program showcased by this [Developer Hub Guide](https://developers.metaplex.com/core/guides/onchain-ticketing-with-appdata).

## Getting Started

### Prequisites
* Node v18.20.4 or higher
* Yarn 1.22.22 or higher
* Rust v1.75.0 or higher
* Anchor CLI 0.30.1 or higher
* Solana CLI 1.18.18 or higher

### Install dependencies
```
yarn
```

### Build the program and get mpl-core
```
anchor build
./dump-programs.sh
```

### Run the tests
```
anchor test
```
