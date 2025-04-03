use serde::{Deserialize, Serialize};

/// TXInput represents a transaction input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
  pub txid: String,
  pub vout: i32,
  pub signature: Vec<u8>,
  pub pub_key: Vec<u8>,
}

/// TXInput represents a transaction output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
  pub value: i32,
  pub pub_key_hash: Vec<u8>,
}

impl TXInput {
  /// CanUnlockOutputWith checks whether the address initiated the transaction
  pub fn can_unlock_output_with(&self, unlocking_data: &str) -> bool {
    self.script_sig == unlocking_data
  }
}

impl TXOutput {
  /// CanBeUnlockedWith checks if the output can be unlocked with the provided data
  pub fn can_be_unlock_with(&self, unlocking_data: &str) -> bool {
    self.script_pub_key == unlocking_data
  }
}