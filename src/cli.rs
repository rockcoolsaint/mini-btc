use clap::{arg, Command};

use crate::blockchain::Blockchain;
use crate::errors::Result;

pub struct Cli {
  bc: Blockchain,
}

impl Cli {
  pub fn new() -> Result<Cli> {
    Ok(Cli {
      bc: Blockchain::new()?
    })
  }
  pub fn run(&mut self) -> Result<()> {
    let matches = Command::new("blockchain-rust-demo")
          .version("0.1")
          .author("behrouz.r.fa@gmail.com")
          .about("blockchain in rust: a simple blockchain for learning")
          .subcommand(Command::new("printchain").about("print all the chain blocks"))
          .subcommand(
            Command::new("addblock")
            .about("add a new block to the chain")
            .arg(arg!(<data>" 'the blockchain data")),
          )
          .get_matches();

        if let Some(ref matches) = matches.subcommand_matches("addblock") {
            if let Some(c) = matches.get_one::<String>("DATA") {
              self.addblock(String::from(c))?;
            } else {
              println!("No printing testing lists...");
            }
        }

      // if let Some(ref matches) = matches.subcommand_matches("startminer") {
      //     let port = if let Some(port) = matches.get_one::<String>("PORT") {
      //         port
      //     } else {
      //         println!("PORT not supply!: usage");
      //         exit(1)
      //     };

      //     let address = if let Some(address) = matches.get_one::<String>("ADDRESS") {
      //         address
      //     } else {
      //         println!("ADDRESS not supply!: usage");
      //         exit(1)
      //     };
      //     let bc = Blockchain::new()?;
      //     let utxo_set = UTXOSet { blockchain: bc };
      //     let server = Server::new(port, address, utxo_set)?;
      //     server.start_server()?;
      // }


      // if let Some(ref matches) = matches.subcommand_matches("startnode") {
      //     if let Some(port) = matches.get_one::<String>("PORT") {
      //         let bc = Blockchain::new()?;
      //         let utxo_set = UTXOSet { blockchain: bc };
      //         let server = Server::new(port, "", utxo_set)?;
      //         server.start_server()?;
      //     }
      // }

      // if let Some(_) = matches.subcommand_matches("createwallet") {
      //     println!("address: {}", cmd_create_wallet()?);
      // }
      // if let Some(_) = matches.subcommand_matches("reindex") {
      //     let count = cmd_reindex()?;
      //     println!("Done! There are {} transactions in the UTXO set.", count);
      // }

      // if let Some(_) = matches.subcommand_matches("listaddresses") {
      //     cmd_list_address()?;
      // }

      // if let Some(ref matches) = matches.subcommand_matches("create") {
      //     if let Some(address) = matches.get_one::<String>("ADDRESS") {
      //         cmd_create_blockchain(address)?;
      //     }

      // }


      // if let Some(ref matches) = matches.subcommand_matches("getbalance") {
      //     if let Some(address) = matches.get_one::<String>("ADDRESS") {
      //         let balance = cmd_get_balance(address)?;
      //         println!("Balance: {}\n", balance);
      //     }
      // }

      // if let Some(ref matches) = matches.subcommand_matches("send") {
      //     let from = if let Some(address) = matches.get_one::<String>("FROM") {
      //         address
      //     } else {
      //         println!("from not supply!: usage");
      //         exit(1)
      //     };

      //     let to = if let Some(address) = matches.get_one::<String>("TO") {
      //         address
      //     } else {
      //         println!("from not supply!: usage");
      //         exit(1)
      //     };

      //     let amount: i32 = if let Some(amount) = matches.get_one::<String>("AMOUNT") {
      //         amount.parse()?
      //     } else {
      //         println!("from not supply!: usage");
      //         exit(1)
      //     };

      //     if matches.contains_id("mine") {
      //         cmd_send(from, to, amount, true)?;
      //     } else {
      //         cmd_send(from, to, amount, false)?;
      //     }


          /*else {
              println!("Not printing testing lists...");
          }*/
      // }

      if let Some(_) = matches.subcommand_matches("printchain") {
          self.print_chain()?;
      }

      Ok(())
  }

  fn addblock(&mut self, data: String) -> Result<()> {
    self.bc.add_block(data)
  }

  fn print_chain(&mut self) {
    for b in &mut self.bc.iter() {
      println!("block: {:?}", b);
    }
  }
}