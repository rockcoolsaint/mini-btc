use bincode::serialize;
use crypto::digest::Digest;
use crypto::ed25519;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use failure::format_err;
use log::error;
use crate::errors::Result;
use crate::utxoset::UTXOSet;
use crate::wallets::{Wallet, Wallets};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::blockchain::Blockchain;
use crate::tx::{TXInput, TXOutput};

/// Transaction represents a Bitcoin transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
  pub id: String,
  pub vin: Vec<TXInput>,
  pub vout: Vec<TXOutput>,
}

impl Transaction {
  /// NewUTXOTransaction creates a new transaction
  pub fn new_UTXO(wallet: &Wallet, to: &str, amount: i32, utxo: &UTXOSet) -> Result<Transaction> {
    let mut vin = Vec::new();

    let mut pub_key_hash = wallet.public_key.clone();
    hash_pub_key(&mut pub_key_hash);

    let acc_v = utxo.find_spendable_outputs(&pub_key_hash, amount)?;

    if acc_v.0 < amount {
      error!("Not Enough balance");
      return Err(format_err!(
          "Not Enough balance: current balance {}",
          acc_v.0
      ));
    }
    
    for tx in acc_v.1 {
        for out in tx.1 {
            let input = TXInput {
                txid: tx.0.clone(),
                vout: out,
                signature: Vec::new(),
                pub_key: wallet.public_key.clone(),
            };
            vin.push(input);
        }
    }

    let mut vout = vec![TXOutput::new(
      amount,
      to.to_string())?];

    if acc_v.0 > amount {
      vout.push(TXOutput::new(
        acc_v.0 - amount,
        wallet.get_address())?)
    }

    let mut tx = Transaction {
        id: String::new(),
        vin,
        vout,
    };
    tx.id = tx.hash()?;
    // utxo.blockchain
    //     .sign_transacton(&mut tx, &wallet.secret_key)?;
  
    Ok(tx)
  }

  pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
    if data == String::from("") {
      data += &format!("Reward to '{}'", to);
    }

    let mut tx = Transaction {
      id: String::new(),
      vin: vec![TXInput {
        txid: String::new(),
        vout: -1,
        signature: Vec::new(),
        pub_key: Vec::from(data.as_bytes())
      }],
      vout: vec![TXOutput::new(100, to)?],
    };
    tx.id = tx.hash()?;
    Ok(tx)
  }

  /// IsCoinbase checks whether the transaction is coinbase
  pub fn is_coinbase(&self) -> bool {
    self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
  }

  /// Verify verifies signatures of Transaction inputs
  pub fn verify(&self, prev_TXs: HashMap<String, Transaction>) -> Result<bool> {
      if self.is_coinbase() {
          return Ok(true);
      }

      for vin in &self.vin {
          if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
              return Err(format_err!("ERROR: Previous transaction is not correct"));
          }
      }

      let mut tx_copy = self.trim_copy();

      for in_id in 0..self.vin.len() {
          let prev_Tx = prev_TXs.get(&self.vin[in_id].txid).unwrap();
          tx_copy.vin[in_id].signature.clear();
          tx_copy.vin[in_id].pub_key = prev_Tx.vout[self.vin[in_id].vout as usize]
              .pub_key_hash
              .clone();
          tx_copy.id = tx_copy.hash()?;
          tx_copy.vin[in_id].pub_key = Vec::new();

          if !ed25519::verify(
              &tx_copy.id.as_bytes(),
              &self.vin[in_id].pub_key,
              &self.vin[in_id].signature,
          ) {
              return Ok(false);
          }
      }

      Ok(true)
  }

  pub fn sign(
    &mut self,
    private_key: &[u8],
    prev_TXs: HashMap<String, Transaction>,
  ) -> Result<()> {
    if self.is_coinbase() {
      return Ok(());
    }

    for vin in &self.vin {
      if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
        return Err(format_err!("Error: Previous transaction is not correct"));
      }
    }
    let mut tx_copy = self.trim_copy();

    for in_id in 0..tx_copy.vin.len() {
        let prev_Tx = prev_TXs.get(&tx_copy.vin[in_id].txid).unwrap();
        tx_copy.vin[in_id].signature.clear();
        tx_copy.vin[in_id].pub_key = prev_Tx.vout[tx_copy.vin[in_id].vout as usize]
            .pub_key_hash
            .clone();
        tx_copy.id = tx_copy.hash()?;
        tx_copy.vin[in_id].pub_key = Vec::new();
        let signature = ed25519::signature(tx_copy.id.as_bytes(), private_key);
        self.vin[in_id].signature = signature.to_vec();
    }

    Ok(())
  }

  /// Hash returns the hash of the Transaction
  pub fn hash(&self) -> Result<String> {
    let mut copy = self.clone();
    copy.id = String::new();
    let data = serialize(&copy)?;
    let mut hasher = Sha256::new();
    hasher.input(&data[..]);
    Ok(hasher.result_str())
  }


  /// TrimmedCopy creates a trimmed copy of Transaction to be used in signing
  fn trim_copy(&self) -> Transaction {
    let mut vin = Vec::new();
    let mut vout = Vec::new();

    for v in &self.vin {
      vin.push(TXInput {
          txid: v.txid.clone(),
          vout: v.vout.clone(),
          signature: Vec::new(),
          pub_key: Vec::new(),
      })
    }

    for v in &self.vout {
        vout.push(TXOutput {
            value: v.value,
            pub_key_hash: v.pub_key_hash.clone(),
        })
    }

    Transaction {
        id: self.id.clone(),
        vin,
        vout,
    }
  }
}

pub fn hash_pub_key(pubKey: &mut Vec<u8>) {
  let mut hasher1 = Sha256::new();
  hasher1.input(pubKey);
  hasher1.result(pubKey);
  let mut hasher2 = Ripemd160::new();
  hasher2.input(pubKey);
  pubKey.resize(20, 0);
  hasher2.result(pubKey);
}