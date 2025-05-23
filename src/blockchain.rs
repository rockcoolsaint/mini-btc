use std::collections::HashMap;

use bincode::{deserialize, serialize};
use failure::format_err;
use log::info;

use crate::block::Block;
use crate::errors::Result;
use crate::transaction::Transaction;
use crate::tx::{TXOutput, TXOutputs};
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

  /// MineBlock mines a new block with the provided transactions
  pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
      info!("mine a new block");

      for tx in &transactions {
          if !self.verify_transacton(tx)? {
              return Err(format_err!("ERROR: Invalid transaction"));
          }
      }

      let lasthash = self.db.get("LAST")?.unwrap();

      let newblock = Block::new_block(
          transactions,
          String::from_utf8(lasthash.to_vec())?,
          self.get_best_height()? + 1,
      )?;
      self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
      self.db.insert("LAST", newblock.get_hash().as_bytes())?;
      self.db.flush()?;

      self.current_hash = newblock.get_hash();
      Ok(newblock)
  }

  /// AddBlock saves the block into the blockchain
  pub fn add_block(&mut self, block: Block) -> Result<()> {
      let data = serialize(&block)?;
      if let Some(_) = self.db.get(block.get_hash())? {
          return Ok(());
      }
      self.db.insert(block.get_hash(), data)?;

      let lastheight = self.get_best_height()?;
      if block.get_height() > lastheight {
          self.db.insert("LAST", block.get_hash().as_bytes())?;
          self.current_hash = block.get_hash();
          self.db.flush()?;
      }
      Ok(())
  }

  /// FindUnspentTransactions returns a list of transactions containing unspent outputs
  fn find_unspent_transactions(&self, address: &[u8]) -> Vec<Transaction> {
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
  pub fn find_UTXO(&self) -> HashMap<String, TXOutputs> {
    let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
    let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();
    for block in self.iter() {
        for tx in block.get_transaction() {
            for index in 0..tx.vout.len() {
                if let Some(ids) = spend_txos.get(&tx.id) {
                    if ids.contains(&(index as i32)) {
                        continue;
                    }
                }

                match utxos.get_mut(&tx.id) {
                    Some(v) => {
                        v.outputs.push(tx.vout[index].clone());
                    }
                    None => {
                        utxos.insert(
                            tx.id.clone(),
                            TXOutputs {
                                outputs: vec![tx.vout[index].clone()],
                            },
                        );
                    }
                }
            }

            if !tx.is_coinbase() {
                for i in &tx.vin {
                    match spend_txos.get_mut(&i.txid) {
                        Some(v) => {
                            v.push(i.vout);
                        }
                        None => {
                            spend_txos.insert(i.txid.clone(), vec![i.vout]);
                        }
                    }
                }
            }
        }
    }

    utxos
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

  // GetBlock finds a block by its hash and returns it
  pub fn get_block(&self, block_hash: &str) -> Result<Block> {
      let data = self.db.get(block_hash)?.unwrap();
      let block = deserialize(&data.to_vec())?;
      Ok(block)
  }

  /// GetBestHeight returns the height of the latest block
  pub fn get_best_height(&self) -> Result<i32> {
      let lasthash = if let Some(h) = self.db.get("LAST")? {
          h
      } else {
          return Ok(-1);
      };
      let last_data = self.db.get(lasthash)?.unwrap();
      let last_block: Block = deserialize(&last_data.to_vec())?;
      Ok(last_block.get_height())
  }

  /// GetBlockHashes returns a list of hashes of all the blocks in the chain
  pub fn get_block_hashs(&self) -> Vec<String> {
      let mut list = Vec::new();
      for b in self.iter() {
          list.push(b.get_hash());
      }
      list
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