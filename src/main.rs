use crate::cli::Cli;
use crate::errors::Result;

mod block;

mod errors;
mod blockchain;
mod cli;
mod transaction;
mod tx;
mod wallets;

fn main() -> Result<()>{
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}
