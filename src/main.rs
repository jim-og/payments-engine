use ledger::Ledger;
use std::env;
use std::io::{Error, ErrorKind};
use types::{Transaction, TransactionEntry};

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

    let mut ledger = Ledger::default();

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(args[1].clone())?;

    for entry in reader.deserialize() {
        let transaction_entry: TransactionEntry = entry?;
        match Transaction::try_from(transaction_entry) {
            Ok(transaction) => {
                if let Err(e) = ledger.update(transaction) {
                    eprintln!("{:?}", e);
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }

    // TODO print client accounts

    Ok(())
}
