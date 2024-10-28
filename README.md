# TAHM-KENCH

[![License](https://img.shields.io/github/license/sota-zk-labs/orn)](./LICENSE)

[//]: # ([![Continuous Integration]&#40;https://github.com/sota-zk/orn/actions/workflows/ci.yaml/badge.svg&#41;]&#40;https://github.com/sota-zk/orn/actions/workflows/ci.yaml/badge.svg&#41;)

## Introduction

**Tahm-Kench** is a **Sealed-Bid Auction** platform built on **SP1 zkVM**,
using [Aligned layer](https://alignedlayer.com/) to verify the proof. Its main objective is to enable secure and private auctions by using **zero-knowledge proof (ZKP)** to select the winning bidder without
revealing any details about individual bids. This ensures complete confidentiality for bidders while maintaining
fairness in determining the highest bid.

We are excited about this project because it applies **zero-knowledge** techniques to real-world challenges, where
privacy, transparency and security are crucial. This platform addresses the challenge of maintaining participant
confidentiality while ensuring a fair outcome in sealed-bid auctions. The potential applications are vast, from
government contracts and corporate procurement to high-value asset auctions, where secure and anonymous bidding is
essential.

Beyond its practical use, this project also serves as a **reference model** for developers interested in building
ZK-based decentralized applications using **SP1 zkVM** and **Aligned layer**. It demonstrates how zero-knowledge technology can enhance privacy
in competitive bidding scenarios and lays a foundation for future projects in the ZK space.

## Instructions

### Requirements

1. [Rust](https://www.rust-lang.org/tools/install)
2. [Foundry](https://getfoundry.sh)
3. [Aligned CLI](https://docs.alignedlayer.com/introduction/1_try_aligned)
4. [SP1](https://docs.succinct.xyz/getting-started/install.html)

### Setup

First, you need to create a local keystore using the `cast` tool.
If you already have one you can skip this step.

```bash
cast wallet import --private-key <YOUR_PRIVATE_KEY>
```

Then, clone our repository:
```bash
# clone the repository
git clone https://github.com/sota-zk-labs/tahm-kench
cd tahm-kench
```

### 2 - Run Cli Tools

To use **Tahm-Kench**, you need run:

```bash
make make start-cli KEYSTORE_PATH=<path_to_keystore.json>
```
