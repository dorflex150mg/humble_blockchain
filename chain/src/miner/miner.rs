use crate::block::block::{self, Block, Hash, InvalidTransactionErr};
use wallet::transaction::transaction::Transaction;
use wallet::wallet::Wallet;

use rand::{self, Rng};
use std::cmp;
use std::fmt;

use thiserror::Error;

pub const ZERO_WALLET_PK: [u8; 64] = [0u8; 64];

#[derive(Clone)]
pub struct ChainMeta {
    pub len: usize,
    pub difficulty: usize,
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MiningDigest {
    block: Block,
    nonce: u64,
}

impl MiningDigest {
    pub fn new(block: Block, nonce: u64) -> Self {
        MiningDigest { block, nonce }
    }

    pub fn get_block(&self) -> Block {
        self.block.clone()
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }
}

#[derive(Error, Debug, derive_more::From, derive_more::Display)]
pub enum MiningError {
    InvalidTransactionErr(InvalidTransactionErr),
    UninitializedChainMetaErr(UninitializedChainMetaErr),
}

#[derive(Error, Debug)]
pub struct UninitializedChainMetaErr;

impl fmt::Display for UninitializedChainMetaErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "This miner's ChainMeta has not been initialized. Consider set_chain_meta(len, last_hash, difficulty)")
    }
}

pub struct Miner {
    id: u64,
    name: String,
    pub wallet: Wallet,
    pub transactions: Vec<Transaction>,
    pub chain_meta: Option<ChainMeta>,
}

impl Miner {
    pub fn new(id: u64, name: String) -> Self {
        Miner {
            id,
            name,
            wallet: Wallet::new(),
            transactions: vec![],
            chain_meta: None,
        }
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn mine(&mut self, mut block: Block) -> Result<MiningDigest, MiningError> {
        self.transactions = self.check_transactions();
        let chain_meta = self
            .chain_meta
            .as_ref()
            .ok_or(MiningError::UninitializedChainMetaErr(
                UninitializedChainMetaErr,
            ))?;
        loop {
            let mut rng = rand::thread_rng();
            block.nonce = rng.gen_range(0..=u64::MAX);
            let str_digest = block.calculate_hash();
            if str_digest.starts_with(&"0".repeat(chain_meta.difficulty)) {
                let prize_transaction = Transaction::new(
                    ZERO_WALLET_PK.to_vec(),
                    self.wallet.get_pub_key(),
                    vec![str_digest.to_string()],
                );
                let signed_prize = self.wallet.sign(prize_transaction);
                self.transactions.push(signed_prize); //TODO: this should be the 1st tx
                return Ok(MiningDigest::new(
                    self.create_new_block(str_digest, block.hash.clone()),
                    block.nonce,
                ));
            } else {
                continue;
            }
        }
    }

    pub fn set_chain_meta(&mut self, len: usize, difficulty: usize, blocks: Vec<Block>) {
        self.chain_meta = Some(ChainMeta {
            len,
            difficulty,
            blocks,
        })
    }

    #[allow(dead_code)]
    pub fn set_transactions(&mut self, new_transactions: Vec<Transaction>) {
        self.transactions = new_transactions;
    }

    pub fn push_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn check_transactions(&self) -> Vec<Transaction> {
        let chain_meta = self
            .chain_meta
            .as_ref()
            .ok_or(MiningError::UninitializedChainMetaErr(
                UninitializedChainMetaErr,
            ))
            .unwrap();
        let filtered: Vec<Transaction> = self
            .transactions
            .iter()
            .filter_map(|transaction| {
                block::check_transaction(transaction.clone(), &chain_meta.blocks).ok()
            })
            .collect();
        filtered
    }

    pub fn create_new_block(&mut self, hash: Hash, previous_hash: Hash) -> Block {
        let index = self.chain_meta.clone().unwrap().len + 1;
        let cap = cmp::min(self.transactions.len(), block::MAX_TRANSACTIONS);
        let capped_transactions: Vec<Transaction> = self.transactions.drain(0..cap).collect();
        let encoded_transactions: Vec<String> = capped_transactions
            .iter()
            .map(|transaction| transaction.clone().into())
            .collect();
        let data = encoded_transactions.join("");
        self.wallet.add_coin(hash.clone().try_into().unwrap());
        Block::new(index, previous_hash, data, Some(hash))
    }
}

impl fmt::Display for Miner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let joint_coins = self
            .wallet
            .coins
            .iter()
            .map(|coin| (*coin).clone().try_into().unwrap())
            .collect::<Vec<String>>()
            .join(",");

        write!(
            f,
            "id: {}, name: {}, wallet: {}",
            self.id, self.name, joint_coins
        )
    }
}
