use std::collections::HashMap;

use failure::format_err;
use log::info;

use crate::block::Block;
use crate::errors::Result;
use crate::transaction::Transaction;
use crate::tx::TXOutput;
// use sled;

const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

const TARGET_HEXT: usize = 4;


#[derive(Debug)]
pub struct Blockchain {
  current_hash: String,
  db: sled::Db
}

pub struct BlockchainIter<'a> {
  current_hash: String,
  bc: &'a Blockchain,
}

impl Blockchain {
  /// NewBlockchain creates a new Blockchain db
  pub fn new() -> Result<Blockchain> {
    info!("open blockchain");

    let db = sled::open("data/blocks")?;
    let hash = db
        .get("LAST")?
        .expect("Must create a new block database first");
      info!("Found block database");
      let lasthash = String::from_utf8(hash.to_vec())?;
      Ok(Blockchain {
        current_hash: lasthash.clone(),
        db,
      })

  }

  /// CreateBlockchain creates a new blockchain DB
  pub fn create_blockchain(address: String) -> Result<Blockchain> {
    info!("Create new blockchain");

    let db = sled::open("data/blocks")?;
    info!("Creating new block databse");
    let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
    let genesis: Block = Block::new_genesis_block(cbtx);
    db.insert(genesis.get_hash(), bincode::serialize(&genesis)?)?;
    db.insert("LAST", genesis.get_hash().as_bytes())?;
    let bc = Blockchain {
      current_hash: genesis.get_hash(),
      db,
    };
    bc.db.flush()?;
    Ok(bc)
  }

  pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
    let lasthash = self.db.get("LAST")?.unwrap();

    let new_block = Block::new_block(transactions, String::from_utf8(lasthash.to_vec())?, TARGET_HEXT)?;
    self.db.insert(new_block.get_hash(), bincode::serialize(&new_block)?)?;
    self.db.insert("LAST", new_block.get_hash().as_bytes())?;
    self.current_hash = new_block.get_hash();
    Ok(())
  }

  /// FindUnspentTransactions returns a list of transactions containing unspent outputs
  fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
    let mut spent_TXOs: HashMap<String, Vec<i32>> = HashMap::new();
    let mut unspend_TXs: Vec<Transaction> = Vec::new();

    for block in self.iter() {
      for tx in block.get_transaction() {
        for index in 0..tx.vout.len() {
          if let Some(ids) = spent_TXOs.get(&tx.id) {
            if ids.contains(&(index as i32)) {
              continue;
            }
          }

          if tx.vout[index].can_be_unlock_with(address) {
            unspend_TXs.push(tx.to_owned())
          }
        }

        if !tx.is_coinbase() {
          for i in &tx.vin {
            if i.can_unlock_output_with (address) {
              match spent_TXOs.get_mut(&i.txid) {
                Some(v) => {
                  v.push(i.vout);
                }
                None => {
                  spent_TXOs.insert(i.txid.clone(), vec![i.vout]);
                }
              }
            }
          }
        }
      }
    }

    unspend_TXs
  }

  /// FindUTXO finds and returns all unspent transaction outputs
  pub fn find_UTXO(&self, address: &str) -> Vec<TXOutput> {
    let mut utxos = Vec::<TXOutput>::new();
    let unspend_TXs = self.find_unspent_transactions(address);
    for tx in unspend_TXs {
      for out in &tx.vout {
        if out.can_be_unlock_with(&address) {
          utxos.push(out.clone());
        }
      }
    }

    utxos
  }

  /// FindUnspentTransactions returns a list of transactions containing unspent outputs
  pub fn find_spendable_outputs(
    &self,
    address: &str,
    amount: i32,
  ) -> (i32, HashMap<String, Vec<i32>>) {
    let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
    let mut accumulated = 0;
    let unspend_TXs = self.find_unspent_transactions(address);

    for tx in unspend_TXs {
      for index in 0..tx.vout.len() {
        if tx.vout[index].can_be_unlock_with(address) && accumulated < amount {
          match unspent_outputs.get_mut(&tx.id) {
            Some(v) => v.push(index as i32),
            None => {
              unspent_outputs.insert(tx.id.clone(), vec![index as i32]);
            }
          }
          accumulated += tx.vout[index].value;

          if accumulated >= amount {
            return (accumulated, unspent_outputs);
          }
        }
      }
    }
    (accumulated, unspent_outputs)
  }

  pub fn iter(&self) -> BlockchainIter {
    BlockchainIter {
      current_hash: self.current_hash.clone(),
      bc: &self,
    }
  }

  /// FindTransaction finds a transaction by its ID
  pub fn find_transacton(&self, id: &str) -> Result<Transaction> {
    for b in self.iter() {
      for tx in b.get_transaction() {
          if tx.id == id {
              return Ok(tx.clone());
          }
      }
    }
    Err(format_err!("Transaction is not found"))
  }

  fn get_prev_TXs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
    let mut prev_TXs = HashMap::new();
    for vin in &tx.vin {
        let prev_TX = self.find_transacton(&vin.txid)?;
        prev_TXs.insert(prev_TX.id.clone(), prev_TX);
    }
    Ok(prev_TXs)
  }

  /// SignTransaction signs inputs of a Transaction
  pub fn sign_transacton(&self, tx: &mut Transaction, private_key: &[u8]) -> Result<()> {
    let prev_TXs = self.get_prev_TXs(tx)?;
    tx.sign(private_key, prev_TXs)?;
    Ok(())
  }

  /// VerifyTransaction verifies transaction input signatures
  pub fn verify_transacton(&self, tx: &Transaction) -> Result<bool> {
    if tx.is_coinbase() {
        return Ok(true);
    }
    let prev_TXs = self.get_prev_TXs(tx)?;
    tx.verify(prev_TXs)
  }
}

impl<'a> Iterator for BlockchainIter<'a> {
  type Item = Block;

  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(encode_block) = self.bc.db.get(&self.current_hash) {
      return match encode_block {
        Some(b) => {
          if let Ok(block) = bincode::deserialize::<Block>(&b) {
            self.current_hash = block.get_prev_hash();
            Some(block)
            } else {
              None
            }
          }
        None => None,
      };
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_blockchain() {
    let mut b = Blockchain::new().unwrap();
    // b.add_block("data".to_string());
    // b.add_block("data2".to_string());
    // b.add_block("data3".to_string());
    // dbg!(b);

    for item in b.iter() {
      println!("item: {:?}", item);
    }
  }
}