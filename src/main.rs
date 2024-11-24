use ledger::Ledger;
use std::env;
use std::fs::File;
use std::io::{Error, ErrorKind};
use utils::read_input;

mod ledger;
mod types;
mod utils;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Error::new(
            ErrorKind::Other,
            "please specify a single input file argument",
        ));
    }

    // Create the ledger which will track client transactions
    let mut ledger = Ledger::default();

    let input_file = File::open(args[1].clone())?;
    for entry in read_input(input_file) {
        match entry {
            Ok(transaction) => {
                if let Err(e) = ledger.update(transaction) {
                    eprintln!("{}", e);
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }

    ledger.print(std::io::stdout())?;
    Ok(())
}
