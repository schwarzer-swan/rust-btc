use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    crypto::PublicKey,
    types::{Block, Transaction, TransactionOutput},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    /// Fetch all UTXOs belonging to a publickey
    FetchUTXOs(PublicKey),
    /// UTXOs belonging to a publickey, bool determines if marked
    UTXOs(Vec<(TransactionOutput, bool)>),
    /// Send a transaction to the network
    SubmitTransaction(Transaction),
    /// Broadcast a new transaction to otner nodes
    NewTransaction(Transaction),
    /// Task node to prepare the optimal block template
    /// with coinbase transaction paying to given publickey
    FetchTemplate(PublicKey),
    /// The template
    Template(Block),
    /// Ask the node to validate a block template.
    ValidateTemplate(Block),
    /// If template is valid
    TemplateValidity(bool),
    /// Submit a mined block to a node
    SubmitTemplate(Block),
    /// Ask a node to report all the other nodes it knows
    /// about
    DiscoverNodes,
    /// This is the response to DiscoverNodes
    NodeList(Vec<String>),
    /// Ask a node whats the highest block it knows about
    /// in comparison to the local blockchain
    AskDifference(u32),
    /// This is the response to AskDifference
    Difference(i32),
    /// Ask a node to send a block with the specified height
    FetchBlock(usize),
    /// Broadcast a new block to other nodes
    NewBlock(Block),
}

// network.rs
// We are going to use length-prefixed encoding for message
// And we are going to use ciborium (CBOR) for serialization
impl Message {
    pub fn encode(&self) -> Result<Vec<u8>, ciborium::ser::Error<IoError>> {
        let mut bytes: Vec<u8> = Vec::new();
        ciborium::into_writer(self, &mut bytes)?;
        Ok(bytes)
    }

    pub fn decode(data: &[u8]) -> Result<Self, ciborium::de::Error<IoError>> {
        ciborium::from_reader(data)
    }

    pub fn send(&self, stream: &mut impl Write) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        let len = bytes.len();
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&bytes)?;
        Ok(())
    }

    pub async fn send_async(
        &self,
        stream: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        let len = bytes.len();
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&bytes).await?;

        Ok(())
    }
    pub fn receive(stream: &mut impl Read) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes)?;
        let len = u64::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;
        Self::decode(&buf)
    }

    pub async fn receive_async(
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes).await?;
        let len = u64::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        Self::decode(&buf)
    }
}
