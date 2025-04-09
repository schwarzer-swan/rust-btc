use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::types::block::Block;
use crate::types::transaction::{Transaction, TransactionInput, TransactionOutput};
use crate::util::{MerkleRoot, Saveable};
use crate::U256;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    // keep the utxos on the chain
    // the bool marks the utxo as reserved by a transaction in mempool
    // init to false,
    utxos: HashMap<Hash, (bool, TransactionOutput)>,
    blocks: Vec<Block>,
    target: U256,
    #[serde(default, skip_serializing)]
    pub mempool: Vec<(DateTime<Utc>, Transaction)>,
}

impl Saveable for Blockchain {
    fn load<I: std::io::Read>(reader: I) -> std::io::Result<(Self)> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialias blockchain"))
    }

    fn save<O: std::io::Write>(&self, writer: O) -> std::io::Result<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialias blockchain"))
    }
}
/*
// fn mempool_sort_key2(&mut self) -> (Transaction -> u64) { todo!( ) }
fn mempool_key(
    utxos: &HashMap<Hash, (bool, TransactionOutput)>,
) -> impl FnMut(&(DateTime<Utc>, Transaction)) -> u64 + use<'_> {
    |(_, tx): &(DateTime<Utc>, Transaction)| {
        let all_inputs: u64 = tx
            .inputs
            .iter()
            .map(|input| {
                utxos
                    .get(&input.pre_transaction_output_hash)
                    .expect("BUG: impossible")
                    .value
            })
            .sum::<u64>();
        let all_outputs: u64 = tx.outputs.iter().map(|output| output.value).sum();
        let miner_fee: u64 = all_inputs - all_outputs;
        miner_fee
    }
}
*/
impl Blockchain {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
            blocks: vec![],
            target: crate::MIN_TARGET,
            mempool: vec![],
        }
    }

    pub fn utxos(&self) -> &HashMap<Hash, (bool, TransactionOutput)> {
        &self.utxos
    }
    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter()
    }
    pub fn target(&self) -> U256 {
        self.target
    }
    pub fn mempool(&self) -> &[(DateTime<Utc>, Transaction)] {
        &self.mempool
    }

    pub fn block_height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// Rebuild utxo set from blockchain
    pub fn rebuild_utoxs(&mut self) {
        for block in &self.blocks {
            for transaction in &block.transactions {
                for input in &transaction.inputs {
                    self.utxos.remove(&input.pre_transaction_output_hash);
                }
                for output in transaction.outputs.iter() {
                    // this will have hash collisions
                    self.utxos
                        .insert(transaction.hash(), (false, output.clone()));
                }
            }
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        if self.blocks.is_empty() {
            if block.header.prev_block_hash != Hash::zero() {
                println!("zero hash");
                return Err(BtcError::InvalidBlock);
            }
        } else {
            let last_block = self.blocks.last().unwrap();
            if block.header.prev_block_hash != last_block.hash() {
                println!("prev hash is wrong");
                return Err(BtcError::InvalidBlock);
            }
            if !block.header.hash().matches_target(block.header.target) {
                return Err(BtcError::InvalidBlock);
            }

            let calculated_merkle_root = MerkleRoot::calculate(&block.transactions);
            if calculated_merkle_root != block.header.merkle_root {
                return Err(BtcError::InvalidMerkleRoot);
            }
            // check if the block's timestamp is after the
            // last block's timestamp
            if block.header.timestamp <= last_block.header.timestamp {
                return Err(BtcError::InvalidBlock);
            }
            // Verify all transactions in the block
            // fails if any transaction fails
            block.verify_transactions(self.block_height(), &self.utxos)?
        }

        // remove transactions from mempool
        let block_transactions: HashSet<_> =
            block.transactions.iter().map(|tx| tx.hash()).collect();

        self.mempool
            .retain(|(_, tx)| !block_transactions.contains(&tx.hash()));

        //
        self.try_adjust_target();
        self.blocks.push(block);
        Ok(())
    }

    pub fn add_to_mempool(&mut self, transaction: Transaction) -> Result<()> {
        // validate transaction before insert
        let mut known_inputs: HashSet<Hash> = HashSet::new();

        // check if any of the utxos have the bool mark set to true
        // and if so, find the transaction that references them
        // in mempool, remove it, and set all the utxos it references
        // to false

        for input in &transaction.inputs {
            if !self.utxos.contains_key(&input.pre_transaction_output_hash) {
                return Err(BtcError::InvalidTransaction);
            }
            if known_inputs.contains(&input.pre_transaction_output_hash) {
                return Err(BtcError::InvalidTransaction);
            }
            known_inputs.insert(input.pre_transaction_output_hash);
        }

        for input in &transaction.inputs {
            if let Some((true, _)) = self.utxos.get(&input.pre_transaction_output_hash) {
                // find the trnaaction that references thi utxo
                let referencing_transaction =
                    self.mempool
                        .iter()
                        .enumerate()
                        .find(|(_, (_, transaction))| {
                            transaction
                                .outputs
                                .iter()
                                .any(|output| output.hash() == input.pre_transaction_output_hash)
                        });
                if let Some((idx, (_, referencing_transaction))) = referencing_transaction {
                    for input in &referencing_transaction.inputs {
                        self.utxos
                            .entry(input.pre_transaction_output_hash)
                            .and_modify(|(marked, _)| {
                                *marked = false;
                            });
                    }
                    self.mempool.remove(idx);
                } else {
                    self.utxos
                        .entry(input.pre_transaction_output_hash)
                        .and_modify(|(marked, _)| {
                            *marked = false;
                        });
                }
            }
        }

        let all_inputs: u64 = transaction
            .inputs
            .iter()
            .map(|input| {
                self.utxos
                    .get(&input.pre_transaction_output_hash)
                    .expect("BUG: impossible")
                    .1
                    .value
            })
            .sum();

        let all_outputs = transaction
            .outputs
            .iter()
            .map(|output| output.value)
            .sum::<u64>();

        if all_inputs < all_outputs {
            return Err(BtcError::InvalidTransaction);
        }
        // mark the utxos as used
        for input in &transaction.inputs {
            self.utxos
                .entry(input.pre_transaction_output_hash)
                .and_modify(|(marked, _)| {
                    *marked = true;
                });
        }

        self.mempool.push((Utc::now(), transaction));
        self.mempool.sort_by_key(|(_, transaction)| {
            let all_inputs: u64 = transaction
                .inputs
                .iter()
                .map(|input| {
                    self.utxos
                        .get(&input.pre_transaction_output_hash)
                        .expect("")
                        .1
                        .value
                })
                .sum::<u64>();
            let all_outputs: u64 = transaction.outputs.iter().map(|output| output.value).sum();
            let miner_fee = all_inputs - all_outputs;
            miner_fee
        });
        Ok(())
    }

    pub fn try_adjust_target(&mut self) {
        if self.block_height() % crate::DIFFICULTY_UPDATE_INTERVAL != 0 {
            return; // not time to adjust the target
        }
        // measure the time it took to mine the last crate::DIFFICULTY_UPDATE_INTERVAL blocks
        let start_time = self.blocks
            [(self.block_height() - crate::DIFFICULTY_UPDATE_INTERVAL) as usize]
            .header
            .timestamp;
        let end_time = self.blocks.last().unwrap().header.timestamp;
        let time_diff = end_time - start_time;
        let time_diff_sconds: Option<i64> = time_diff.num_nanoseconds();
        let target_seconds: u64 = crate::DIFFICULTY_UPDATE_INTERVAL * crate::IDEAL_BLOCK_TIME;
        let new_target = BigDecimal::parse_bytes(&self.target.to_string().as_bytes(), 10)
            .expect("BUG: impossible")
            * (BigDecimal::from(time_diff_sconds.unwrap()) / BigDecimal::from(target_seconds));
        // clamp new_target to be within the range of
        // 4 * self.target and self.target / 4
        let new_target_str = new_target
            .to_string()
            .split('.')
            .next()
            .expect("BUG: Expected a decimal point")
            .to_owned();
        let new_target: U256 = U256::from_str_radix(&new_target_str, 10).expect("BUG: impossible");
        if new_target < self.target / 4 {
            dbg!(self.target / 4)
        } else if new_target > self.target * 4 {
            dbg!(self.target * 4)
        } else {
            new_target
        };

        dbg!(new_target);

        // if the new target is more than the minimum target,
        // set it to the minimum target
        self.target = new_target.min(crate::MIN_TARGET);
        dbg!(self.target);
    }
}
