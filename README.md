# humble_blockchain

## Project Overview

This project implements a simple blockchain environment in Rust. It includes core components like mining, wallet management, transaction processing, and blockchain management. The `test_core` module demonstrates the creation of a blockchain, the mining of blocks by multiple miners, and the handling of transactions between wallets.

## Features

- **Blockchain Creation**: A blockchain is initialized with a genesis block.
- **Mining**: Blocks are mined with or without transactions, adjusting the blockchain length and difficulty dynamically.
- **Wallets**: Each miner has an associated wallet with coins that can be transferred between users.
- **Transactions**: Transactions are signed by wallets and included in the mining process.
- **Concurrency**: Multiple miners can mine simultaneously using threads, with access to a shared blockchain through synchronization mechanisms.

## Structure

The code in the project is broken down into the following main components:

1. **Chain**: Manages the blockchain itself, including adding blocks and managing blockchain metadata.
2. **Miner**: Represents a miner that participates in the mining process, handles wallet interactions, and mines new blocks.
3. **Wallet**: Represents a user's wallet, allowing users to hold coins and sign transactions.
4. **Transaction**: Represents a transaction between two wallets that can be included in a block.

## Usage

### Prerequisites

Before running the project, ensure that you have [Rust](https://www.rust-lang.org/tools/install) installed.

### Running the Project

To execute the blockchain simulation, run the following command in the root directory of your project:

```bash
cargo run
```

This will simulate a blockchain where:

- A blockchain is created with a genesis block.
- Two miners (`miner 1` and `miner 2`) are created.
- Blocks are mined and added to the blockchain.
- A simple transaction is made from `miner 1` to another wallet.
- Mining continues, both with and without transactions, by `miner 1` and `miner 2`.

## Up Next

We are currently working on a gossip protocol and a distributed implementation of the blockchain.


