# TAHM-KENCH

[![License](https://img.shields.io/github/license/sota-zk-labs/orn)](./LICENSE)

[//]: # ([![Continuous Integration]&#40;https://github.com/sota-zk/orn/actions/workflows/ci.yaml/badge.svg&#41;]&#40;https://github.com/sota-zk/orn/actions/workflows/ci.yaml/badge.svg&#41;)

## Introduction

**Tahm-Kench** is a [**Sealed-Bid Auction**](https://www.investopedia.com/terms/s/sealed-bid-auction.asp) platform built using the
**Plonky3** toolkit. The project aims to facilitate secure and private auctions by leveraging **zero-knowledge proofs (ZKPs)** to
determine the highest bidder without revealing individual bid amounts. This ensures both privacy and fairness in the bidding process.

Additionally, **SilentBid** serves as a **reference model** for developers interested in building decentralized applications (dApps)
using **Plonky3** and **ZKPs**.

## Requirements

1. [Rust](https://www.rust-lang.org/tools/install)
2. [Foundry](https://getfoundry.sh)

## Usage

## Usage

### 1 - Create Keystore

You can use cast to create a local keystore.
If you already have one you can skip this step.

```bash
cast wallet new-mnemonic
```

Then you can import your created keystore using:

```bash
cast wallet import --interactive <path_to_keystore.json>
```

Then you need to obtain some funds to pay for gas and proof verification.
You can do this by using this [faucet](https://cloud.google.com/application/web3/faucet/ethereum/holesky)

### 2 - Run Cli Tools

To use **Tahm-Kench**, you need run:
```bash
make make start-cli KEYSTORE_PATH=<path_to_keystore.json>
```
