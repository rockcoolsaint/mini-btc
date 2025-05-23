use std::collections::HashMap;

use bitcoincash_addr::{Address, HashType, Scheme};
use crypto::ripemd160::Ripemd160;
use crypto::{digest::Digest, sha2::Sha256};
use crypto::ed25519;
use log::info;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
  pub secret_key: Vec<u8>,
  pub public_key: Vec<u8>,
}

impl Wallet {
  pub fn new() -> Self {
    let mut key: [u8; 32] = [0; 32];
    OsRng.fill_bytes(&mut key);
    let (secret_key, public_key) = ed25519::keypair(&key);
    let secret_key = secret_key.to_vec();
    let public_key = public_key.to_vec();
    Wallet {
      secret_key,
      public_key,
    }
  }

  pub fn get_address(&self) -> String {
    let mut pub_hash = self.public_key.clone();
    hash_pub_key(&mut pub_hash);
    let address = Address {
      body: pub_hash,
      scheme: Scheme::Base58,
      hash_type: HashType::Script,
      ..Default::default()
    };
    // 0 0 1 I
    address.encode().unwrap()
  }
}

/// HashPubKey hashes public key
pub fn hash_pub_key(pubKey: &mut Vec<u8>) {
  let mut hasher1 = Sha256::new();
  hasher1.input(pubKey);
  hasher1.result(pubKey);
  let mut hasher2 = Ripemd160::new();
  hasher2.input(pubKey);
  pubKey.resize(20, 0);
  hasher2.result(pubKey);
}

pub struct Wallets {
  wallets: HashMap<String, Wallet>,
}

use crate::errors::Result;
impl Wallets {
  pub fn new() -> Result<Wallets> {
    let mut wlt = Wallets {
      wallets: HashMap::<String, Wallet>::new(),
    };

    let db = sled::open("data/wallets")?;
    for item in db.into_iter() {
      let i = item?;
      let address = String::from_utf8(i.0.to_vec())?;
      let wallet = bincode::deserialize(&i.1.to_vec())?;
      wlt.wallets.insert(address, wallet);
    }
    drop(db);
    Ok(wlt)
  }

  pub fn create_wallet(&mut self) -> String {
    let wallet = Wallet::new();
    let address = wallet.get_address();
    self.wallets.insert(address.clone(), wallet);
    info!("Create wallet: {}", address);
    address
  }

  pub fn get_all_addresses(&self) -> Vec<String> {
    let mut addresses = Vec::new();
    for (address, _) in &self.wallets {
      addresses.push(address.clone())
    }
    addresses
  }

  pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
    self.wallets.get(address)
  }

  pub fn save_all(&self) -> Result<()> {
    let db = sled::open("data/wallets")?;

    for (address, wallet) in &self.wallets {
      let data = bincode::serialize(wallet)?;
      db.insert(address, data)?;
    }

    db.flush()?;
    drop(db);
    Ok(())
  }
}