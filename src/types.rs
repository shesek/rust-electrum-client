//! Return types
//!
//! This module contains definitions of all the complex data structures that are returned by calls

use bitcoin::blockdata::block;
use bitcoin::hashes::hex::FromHex;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::{Script, Txid};

use serde::{de, Deserialize, Serialize};

static JSONRPC_2_0: &str = "2.0";

#[derive(Serialize, Clone)]
#[serde(untagged)]
/// A single parameter of a [`Request`](struct.Request.html)
pub enum Param {
    /// Integer parameter
    U32(u32),
    /// Integer parameter
    Usize(usize),
    /// String parameter
    String(String),
    /// Boolean parameter
    Bool(bool),
    /// JSON parameter
    Json(serde_json::Value),
}

#[derive(Serialize, Clone)]
/// A request that can be sent to the server
pub struct Request<'a> {
    jsonrpc: &'static str,

    /// The JSON-RPC request id
    pub id: usize,
    /// The request method
    pub method: &'a str,
    /// The request parameters
    pub params: Vec<Param>,
}

impl<'a> Request<'a> {
    /// Creates a new request with a default id
    pub fn new(method: &'a str, params: Vec<Param>) -> Self {
        Self {
            id: 0,
            jsonrpc: JSONRPC_2_0,
            method,
            params,
        }
    }

    /// Creates a new request with a user-specified id
    pub fn new_id(id: usize, method: &'a str, params: Vec<Param>) -> Self {
        let mut instance = Self::new(method, params);
        instance.id = id;

        instance
    }
}

/// Format used by the Electrum server to identify an address. The reverse sha256 hash of the
/// scriptPubKey. Documented [here](https://electrumx.readthedocs.io/en/latest/protocol-basics.html#script-hashes).
pub type ScriptHash = [u8; 32];
/// Binary blob that condenses all the activity of an address. Used to detect changes without
/// having to compare potentially long lists of transactions.
pub type ScriptStatus = [u8; 32];

/// Trait used to convert a struct into the Electrum representation of an address
pub trait ToElectrumScriptHash {
    /// Transforms the current struct into a `ScriptHash`
    fn to_electrum_scripthash(&self) -> ScriptHash;
}

impl ToElectrumScriptHash for Script {
    fn to_electrum_scripthash(&self) -> ScriptHash {
        let mut result = sha256::Hash::hash(self.as_bytes()).into_inner();
        result.reverse();

        result
    }
}

fn from_hex<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromHex,
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_hex(&s).map_err(de::Error::custom)
}

fn from_hex_array<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: FromHex + std::fmt::Debug,
    D: de::Deserializer<'de>,
{
    let arr = Vec::<String>::deserialize(deserializer)?;

    let results: Vec<Result<T, _>> = arr
        .into_iter()
        .map(|s| T::from_hex(&s).map_err(de::Error::custom))
        .collect();

    let mut answer = Vec::new();
    for x in results.into_iter() {
        answer.push(x?);
    }

    Ok(answer)
}

fn from_hex_header<'de, D>(deserializer: D) -> Result<block::BlockHeader, D::Error>
where
    D: de::Deserializer<'de>,
{
    use bitcoin::consensus::deserialize;

    let vec: Vec<u8> = from_hex(deserializer)?;
    deserialize(&vec).map_err(de::Error::custom)
}

/// Response to a [`script_get_history`](../client/struct.Client.html#method.script_get_history) request.
#[derive(Debug, Deserialize)]
pub struct GetHistoryRes {
    /// Confirmation height of the transaction. 0 if unconfirmed, -1 if unconfirmed while some of
    /// its inputs are unconfirmed too.
    pub height: i32,
    /// Txid of the transaction.
    pub tx_hash: Txid,
    /// Fee of the transaction.
    pub fee: Option<u64>,
}

/// Response to a [`script_list_unspent`](../client/struct.Client.html#method.script_list_unspent) request.
#[derive(Debug, Deserialize)]
pub struct ListUnspentRes {
    /// Confirmation height of the transaction that created this output.
    pub height: usize,
    /// Txid of the transaction
    pub tx_hash: Txid,
    /// Index of the output in the transaction.
    pub tx_pos: usize,
    /// Value of the output.
    pub value: u64,
}

/// Response to a [`server_features`](../client/struct.Client.html#method.server_features) request.
#[derive(Debug, Deserialize)]
pub struct ServerFeaturesRes {
    /// Server version reported.
    pub server_version: String,
    /// Hash of the genesis block.
    #[serde(deserialize_with = "from_hex")]
    pub genesis_hash: [u8; 32],
    /// Minimum supported version of the protocol.
    pub protocol_min: String,
    /// Maximum supported version of the protocol.
    pub protocol_max: String,
    /// Hash function used to create the [`ScriptHash`](type.ScriptHash.html).
    pub hash_function: Option<String>,
    /// Pruned height of the server.
    pub pruning: Option<i64>,
}

/// Response to a [`server_features`](../client/struct.Client.html#method.server_features) request.
#[derive(Debug, Deserialize)]
pub struct GetHeadersRes {
    /// Maximum number of headers returned in a single response.
    pub max: usize,
    /// Number of headers in this response.
    pub count: usize,
    /// Raw headers concatenated. Normally cleared before returning.
    #[serde(rename(deserialize = "hex"), deserialize_with = "from_hex")]
    pub raw_headers: Vec<u8>,
    /// Array of block headers.
    #[serde(skip)]
    pub headers: Vec<block::BlockHeader>,
}

/// Response to a [`script_get_balance`](../client/struct.Client.html#method.script_get_balance) request.
#[derive(Debug, Deserialize)]
pub struct GetBalanceRes {
    /// Confirmed balance in Satoshis for the address.
    pub confirmed: u64,
    /// Unconfirmed balance in Satoshis for the address.
    pub unconfirmed: u64,
}

/// Response to a [`transaction_get_merkle`](../client/struct.Client.html#method.transaction_get_merkle) request.
#[derive(Debug, Deserialize)]
pub struct GetMerkleRes {
    /// Height of the block that confirmed the transaction
    pub block_height: usize,
    /// Position in the block of the transaction.
    pub pos: usize,
    /// The merkle path of the transaction.
    #[serde(deserialize_with = "from_hex_array")]
    pub merkle: Vec<[u8; 32]>,
}

/// Notification of a new block header
#[derive(Debug, Deserialize)]
pub struct HeaderNotification {
    /// New block height.
    pub height: usize,
    /// Newly added header.
    #[serde(rename = "hex", deserialize_with = "from_hex_header")]
    pub header: block::BlockHeader,
}

/// Notification of the new status of a script
#[derive(Debug, Deserialize)]
pub struct ScriptNotification {
    /// Address that generated this notification.
    pub scripthash: ScriptHash,
    /// The new status of the address.
    pub status: ScriptStatus,
}

/// Errors
#[derive(Debug)]
pub enum Error {
    /// Wraps `std::io::Error`
    IOError(std::io::Error),
    /// Wraps `serde_json::error::Error`
    JSON(serde_json::error::Error),
    /// Wraps `bitcoin::hashes::hex::Error`
    Hex(bitcoin::hashes::hex::Error),
    /// Error returned by the Electrum server
    Protocol(serde_json::Value),
    /// Error during the deserialization of a Bitcoin data structure
    Bitcoin(bitcoin::consensus::encode::Error),
    /// Already subscribed to the notifications of an address
    AlreadySubscribed(ScriptHash),
    /// Not subscribed to the notifications of an address
    NotSubscribed(ScriptHash),
    /// Error during the deserialization of a response from the server
    InvalidResponse(serde_json::Value),
    /// Generic error with a message
    Message(String),
    /// Invalid domain name for an SSL certificate
    InvalidDNSNameError(String),
    /// Missing domain while it was explicitly asked to validate it
    MissingDomain,

    #[cfg(feature = "use-openssl")]
    /// Invalid OpenSSL method used
    InvalidSslMethod(openssl::error::ErrorStack),
    #[cfg(feature = "use-openssl")]
    /// SSL Handshake failed with the server
    SslHandshakeError(openssl::ssl::HandshakeError<std::net::TcpStream>),
}

macro_rules! impl_error {
    ( $from:ty, $to:ident ) => {
        impl std::convert::From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::$to(err)
            }
        }
    };
}

impl_error!(std::io::Error, IOError);
impl_error!(serde_json::Error, JSON);
impl_error!(bitcoin::hashes::hex::Error, Hex);
impl_error!(bitcoin::consensus::encode::Error, Bitcoin);
