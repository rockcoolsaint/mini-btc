/// Transaction represents a Bitcoin transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
  pub id: String,
  pub vin: Vec<TXInput>,
  pub vout: Vec<TXOutput>,
}