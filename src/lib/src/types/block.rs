use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::types::transaction::{Transaction, TransactionOutput};
use crate::util::{MerkleRoot, Saveable};
use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    /// when the block was created
    pub timestamp: DateTime<Utc>,
    /// Used to mine the block
    pub nonce: u64,
    pub prev_block_hash: Hash,
    /// hash of the merkle tree root derived form all the Trnasactions in this block
    pub merkle_root: MerkleRoot,
    /// a number which has to be higher than this block's hash
    pub target: U256,
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {
        Self {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }

    pub fn mine(&mut self, steps: usize) -> bool {
        if self.hash().matches_target(self.target) {
            return true;
        }
        for _ in 0..steps {
            if let Some(new_nonce) = self.nonce.checked_add(1) {
                self.nonce = new_nonce;
            } else {
                self.nonce = 0;
                self.timestamp = Utc::now();
            }
            if self.hash().matches_target(self.target) {
                return true;
            }
        }
        return false;
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Saveable for Block {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialize Block"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Block"))
    }
}
impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn calculate_miner_fee(
        &self,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<u64> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        let mut outputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        // Check every transaction after coinbase
        for transaction in self.transactions.iter().skip(1) {
            for input in &transaction.inputs {
                let prev_output = utxos
                    .get(&input.pre_transaction_output_hash)
                    .map(|(_, output)| output);

                if prev_output.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output = prev_output.unwrap();
                if inputs.contains_key(&input.pre_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction);
                }
                inputs.insert(input.pre_transaction_output_hash, prev_output.clone());
            }
            for output in &transaction.outputs {
                if outputs.contains_key(&output.hash()) {
                    return Err(BtcError::InvalidTransaction);
                }
                outputs.insert(output.hash(), output.clone());
            }
        }

        //
        let input_value: u64 = inputs.values().map(|output| output.value).sum();
        let output_value: u64 = outputs.values().map(|output| output.value).sum();
        Ok(input_value - output_value)
    }

    pub fn verify_coinbase_transaction(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<()> {
        let coinbase_transaction = &self.transactions[0];
        if coinbase_transaction.inputs.len() != 0 {
            return Err(BtcError::InvalidTransaction);
        }
        if coinbase_transaction.outputs.len() == 0 {
            return Err(BtcError::InvalidTransaction);
        }
        let miner_fee = self.calculate_miner_fee(utxos)?;
        let block_reward = crate::INITIAL_REWARD * 10u64.pow(8)
            / 2u64.pow((predicted_block_height / crate::HALVING_INTERVAL) as u32);
        let total_coinbase_outputs: u64 = coinbase_transaction
            .outputs
            .iter()
            .map(|output| output.value)
            .sum();
        if total_coinbase_outputs != block_reward + miner_fee {
            return Err(BtcError::InvalidTransaction);
        }
        Ok(())
    }

    /// verify transaction inputs in the tx are:
    /// 1) on the block
    /// 2) have not been spent
    pub fn verify_transactions(
        &self,
        block_height: u64,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<()> {
        if (&self.transactions).is_empty() {
            return Err(BtcError::InvalidTransaction);
        }
        self.verify_coinbase_transaction(block_height, utxos)?;

        let mut input_hashes: HashSet<Hash> = HashSet::new();

        for transaction in self.transactions.iter().skip(1) {
            // skip the coinbase tx
            let mut input_value = 0;
            let mut output_value = 0;

            for input in &transaction.inputs {
                let prev_output: Option<&TransactionOutput> = utxos
                    .get(&input.pre_transaction_output_hash)
                    .map(|(_, output)| output);
                if prev_output.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output: &TransactionOutput = prev_output.unwrap();
                // no double spending
                if input_hashes.contains(&input.pre_transaction_output_hash) {
                    return Err(BtcError::InvalidTransactionInput);
                }
                // check if signature is valid
                if !input
                    .signature
                    .verify(&input.pre_transaction_output_hash, &prev_output.pubkey)
                {
                    return Err(BtcError::InvalidTransactionInput);
                }
                input_value += prev_output.value;
                //keep track of inputs we've seen
                input_hashes.insert(input.pre_transaction_output_hash.clone());
            }
            for output in &transaction.outputs {
                output_value += output.value
            }
            // delta is the transaction fee
            if input_value < output_value {
                return Err(BtcError::InvalidTransactionInput);
            }
        }
        Ok(())
    }
}
