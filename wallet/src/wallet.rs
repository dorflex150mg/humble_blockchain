use crate::block_chain::{BlockChain, BlockChainBlock};
use crate::token::{Token, TOKEN_SIZE};
use crate::transaction::block_entry_common::Sign;
use crate::transaction::transaction::Transaction;

use thiserror::Error;

use sha2::{Digest, Sha256};

use ring::rand::SystemRandom;
use ring::signature::{self, EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};
use std::fmt;

/// A `Wallet` consists of a `[EcdsaKeyPair]`, utilized to generate its public key and to verify
/// signatures, as well as copy of the tokens it possesses.
pub struct Wallet {
    /// The `EcdsaKeyPair` used to sign and verify `[BlockChainBlock]`.
    pub key_pair: EcdsaKeyPair,
    /// A collection of tokens onwed by this wallet.
    pub coins: Vec<Token>,
    rng: SystemRandom,
}

#[derive(Debug, Error)]
/// Error type for transactions attempts.
pub enum TransactionErr {
    /// Sender did not have sufficient balance.
    #[error(
        "The Transaction requires an amount of tokens greater than this Wallet has available."
    )]
    InsuficientBalance,
    /// Transaction of 0 tokens.
    #[error("The Transaction token amount must be greater than 0.")]
    ZeroAmount,
    /// Token in transaction has wrong encoding or size.
    #[error("Token used in this transaction, is not valid.")]
    InvalidToken,
    /// Token is not owned by the sender.
    #[error("The last owner of Token {0} is not this transaction's spender.")]
    IncompleteChain(String),
}

#[derive(Debug, Error)]
/// Error type for Chain verification.
pub enum ChainVerificationError {
    /// Signature verification error from `[SignatureError]`.
    #[error("{0}")]
    SignatureError(#[from] SignatureError),
    /// The `Block.data` field's hash does not match the its hash field.
    #[error("Block check error. Expected hash {expected}, but got {got}")]
    BlockCheckError {
        /// Hash field value.
        expected: String,
        /// Calculated hash data.
        got: String,
    },
    /// Transaction verification error from `[TransactionErr]`.
    #[error("{0}")]
    TransactionErr(#[from] TransactionErr),
}

#[derive(Debug, Error)]
/// Error type for Signature verification.
pub enum SignatureError {
    #[error("Verification for key {0:?} failed.")]
    /// Signature verification failed.
    VerificationError(Vec<u8>),
    /// No Signature present.
    #[error("Block Entry {0} has no Signature.")]
    NoSignatureError(String),
}

#[allow(clippy::unwrap_used)]
fn generate_key_pair() -> (EcdsaKeyPair, SystemRandom) {
    let rng = SystemRandom::new();
    let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
    let key_pair =
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8_bytes.as_ref(), &rng)
            .unwrap();
    (key_pair, rng)
}

impl Wallet {
    /// Instantiates a new `Wallet`;
    #[must_use]
    pub fn new() -> Self {
        let (key_pair, rng) = generate_key_pair();
        Wallet {
            coins: vec![],
            key_pair,
            rng,
        }
    }

    /// Generates the public key from the `Wallet`'s key pair.
    #[must_use]
    pub fn get_pub_key(&self) -> Vec<u8> {
        self.key_pair.public_key().as_ref().to_vec().clone()
    }

    /// Pushes a token into the `Wallet`.
    pub fn add_coin(&mut self, coin: Token) {
        self.coins.push(coin);
    }

    /// Returns a copy of the coins.
    #[allow(dead_code)]
    #[must_use]
    pub fn get_coins(&self) -> Vec<Token> {
        self.coins.clone()
    }

    #[allow(dead_code)]
    fn check_balance(&self, amount: usize) -> Result<(), TransactionErr> {
        if amount > self.coins.len() {
            return Err(TransactionErr::InsuficientBalance);
        }
        Ok(())
    }

    /// Signs a `[Sign]` object and returns the same value.
    #[allow(clippy::unwrap_used)]
    pub fn sign<T: Sign>(&self, mut entry: T) -> T {
        let vec = entry.get_payload();
        let bytes = &vec;
        entry.set_signature(
            self.key_pair
                .sign(&self.rng, bytes)
                .unwrap()
                .as_ref()
                .to_vec(),
        );
        entry
    }

    /// Verifies the signature in a `[Sign]` object.
    pub fn verify<T: Sign>(
        &self,
        entry: &T,
        pub_key: Option<Vec<u8>>,
    ) -> Result<(), SignatureError> {
        let key = match pub_key {
            Some(k) => k,
            None => self.key_pair.public_key().as_ref().to_vec(),
        };
        if let Some(signature) = entry.get_signature() {
            let unparsed_pk: &[u8] = key.as_slice();
            let peer_public_key =
                signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_ASN1, unparsed_pk);
            return peer_public_key
                .verify(entry.get_payload().as_ref(), signature.as_ref())
                .map_err(|_| SignatureError::VerificationError(signature));
        }
        Err(SignatureError::NoSignatureError(entry.to_string()))
    }

    /// Verifies a `[BlockChain]` object.
    /// 1 - Verifies that all block's hashes match the block's data.
    /// 2 - Verifies the continuity of the chain, i.e., that each block is followed by another that
    ///   references it.
    /// 3 - Goes through each `[Block]`'s `[Transaction]`s and verifies that they are signed by their
    ///   respective senders and verifies that the senders own the `[Tokens]`they have spent.
    /// 4 - Verifies that `[Record]`s are signed by their respective senders.
    pub fn verify_chain(&self, chain: &dyn BlockChain) -> Result<(), ChainVerificationError> {
        let last_block = &chain.get_last_block();
        let mut previous_block_hash = last_block.get_previous_hash();
        let blocks_copy = chain.get_blocks();
        for (index, block) in chain.get_blocks().iter().rev().enumerate() {
            let mut hasher = Sha256::new();
            // Step 1: Verify that this block's data hash matches the field.
            hasher.update(block.get_data());
            let owned_digest = format!("{:x}", hasher.finalize());
            let digest_str = owned_digest.as_str();
            if digest_str != block.get_hash() {
                return Err(ChainVerificationError::BlockCheckError {
                    expected: digest_str.to_owned(),
                    got: block.get_hash().to_owned(),
                });
            }
            // Step 2: Verify that this block's is the one referenced by the next one.
            if index != 0 {
                // skip Step 2 for the last block, since there is no previous hash referencing it.
                if digest_str != previous_block_hash {
                    return Err(ChainVerificationError::BlockCheckError {
                        expected: previous_block_hash.to_owned(),
                        got: digest_str.to_owned(),
                    });
                }
            }
            previous_block_hash = block.get_hash();
            // Step 3: Verify that this block's transactions signatures are correct.
            let transactions = block.get_transactions();
            for transaction in transactions {
                let pk = transaction.get_sender_pk();
                if let Err(e) = self.verify(&transaction, Some(pk)) {
                    return Err(ChainVerificationError::SignatureError(e));
                }
                if let Err(e) = Self::check_transaction_tokens(&transaction, blocks_copy.as_slice())
                {
                    return Err(ChainVerificationError::TransactionErr(e));
                }
            }
            // Step 4: Verify that this block's records signatures are correct.
            let records = block.get_records();
            for record in records {
                if let Err(e) = self.verify(&record, Some(record.get_sender_pk())) {
                    return Err(ChainVerificationError::SignatureError(e));
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code, clippy::unwrap_used)]
    /// Creates a `[Sign]` from this `[Wallet]` to a receiver, identified by its public key.
    pub fn submit_block_entry(
        &mut self,
        receiver: Vec<u8>,
        amount: usize,
    ) -> Result<impl Sign, TransactionErr> {
        if amount == 0 {
            return Err(TransactionErr::ZeroAmount);
        }
        self.check_balance(amount)?;
        let coin_res: Vec<String> = (0..amount)
            .map(|_| self.coins.pop().unwrap())
            .map(|coin| {
                String::from_utf8((*coin).to_vec()).map_err(|_| TransactionErr::InvalidToken)
            })
            .collect::<Result<Vec<String>, _>>()?;
        let coins = coin_res
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

        Ok(self.sign(Transaction::new(
            self.key_pair.public_key().as_ref().to_vec(),
            receiver,
            coins,
        )))
    }

    /// Validates a transaction by checking that the sender owns the coins they are trying to spend.
    ///
    /// # Arguments
    /// * `block_member` - The transaction to validate.
    /// * `blocks` - A slice of blocks that constitute the current blockchain.
    ///
    /// # Returns
    /// * `Result<Transaction, InvalidTransactionErr>` - Returns the validated transaction if successful, or an error if validation fails.
    pub fn check_transaction_tokens(
        transaction: &Transaction,
        blocks: &[Box<dyn BlockChainBlock>],
    ) -> Result<(), TransactionErr> {
        let tokens: &Vec<String> = &transaction.coins;
        for token in tokens {
            //verify each coin is valid:
            let mut coin_found: bool = false;
            for block in blocks.iter().rev() {
                //check each block
                for t in block.get_transactions() {
                    //check each transaction in the block
                    if t.coins[0] == *token {
                        coin_found = true; //if the coin gets found, check if the spender is
                                           //the last owner of the coin
                        if t.receiver_pk != transaction.get_sender_pk() {
                            // fail if sender doesnt own the
                            // coin
                            return Err(TransactionErr::IncompleteChain(token.into()));
                        }
                        break;
                    }
                }
            }
            if !coin_found {
                // if the coin is not in any blocks, fail
                return Err(TransactionErr::InvalidToken);
            }
        }
        Ok(())
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let joint_coins: String = self
            .coins
            .iter()
            .map(|coin| String::from_utf8((*coin).to_vec()).unwrap_or("*".repeat(TOKEN_SIZE)))
            .collect::<Vec<String>>()
            .join(", ");
        let pub_key = self.get_pub_key();
        write!(f, "{pub_key:?}: {joint_coins}")
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Wallet::new()
    }
}
