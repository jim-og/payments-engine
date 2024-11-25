use ledger::Ledger;
use std::env;
use std::fs::File;
use std::io::{Error, ErrorKind};

mod ledger;
mod types;
mod utils;

fn main() -> Result<(), Error> {
    // Ensure there is a single command line argument specifying the input file.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Error::new(
            ErrorKind::Other,
            "please specify a single input file argument",
        ));
    }

    // Attempt to open the specified file.
    let input_file = File::open(args[1].clone())?;

    // Create a ledger to track client transactions.
    let mut ledger = Ledger::default();

    // Load transactions into the ledger.
    ledger.load(input_file);

    // Print client accounts to stdout.
    ledger.print(std::io::stdout())?;
    Ok(())
}
