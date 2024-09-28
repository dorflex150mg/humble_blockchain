# humble_blockchain

An unassuming blockchain implementation

# Blockchain Simulation in Rust

## Project Overview

This project simulates a simple blockchain environment in Rust. It includes core components like mining, wallet management, transaction processing, and blockchain management. The `test_core` module demonstrates the creation of a blockchain, the mining of blocks by multiple miners, and the handling of transactions between wallets.

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

This will simulate a blockchain where:

- A blockchain is created with a genesis block.
- Two miners (`miner 1` and `miner 2`) are created.
- Blocks are mined and added to the blockchain.
- A simple transaction is made from `miner 1` to another wallet.
- Mining continues, both with and without transactions, by `miner 1` and `miner 2`.

### Key Functions

1. **`Chain::new()`**: Creates a new blockchain with a genesis block.
2. **`Miner::new()`**: Creates a new miner with an ID and name.
3. **`miner.mine()`**: Mines a block, optionally including transactions.
4. **`my_chain.add_block()`**: Adds a mined block to the blockchain.
5. **`Transaction::new()`**: Creates a new transaction between two wallets.

### Example

The following example shows the basic steps involved in running the blockchain simulation:

```rust
let mut my_chain = Chain::new(); // Create a blockchain with a genesis block
let mut miner = Miner::new(1, String::from("miner 1")); // Initialize a miner

let wallet1 = Wallet::new(); // Create a new wallet
let last_block = my_chain.get_last_block();

// Miner mines a block
miner.set_chain_meta(my_chain.get_len(), my_chain.difficulty, my_chain.get_blocks());
let (new_block, nonce) = miner.mine(last_block, vec![]).unwrap();
my_chain.add_block(new_block, nonce);

// Miner signs a transaction and mines another block with it
let t1 = Transaction::new(miner.wallet.get_pub_key(), wallet1.get_pub_key(), vec![coin]);
let signed_t1 = miner.wallet.sign(t1);
let (newer_block, new_nonce) = miner.mine(my_chain.get_last_block(), vec![signed_t1]).unwrap();
my_chain.add_block(newer_block, new_nonce);



