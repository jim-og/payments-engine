use ledger::Ledger;
use std::env;
use std::io::{Error, ErrorKind};
use types::Transaction;

mod ledger;
mod types;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Error::new(
            ErrorKind::Other,
            "please specify a single input file argument",
        ));
    }

    let ledger = Ledger::default();

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(args[1].clone())?;

    for row in reader.deserialize() {
        let transaction: Transaction = row?;
        if let Err(e) = ledger.update(transaction) {
            eprintln!("{:?}", e);
        }
    }

    // TODO print client accounts

    Ok(())
}
