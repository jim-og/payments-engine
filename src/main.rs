use rust_decimal::Decimal;
use serde::Deserialize;
use std::env;
use std::io::{Error, ErrorKind};

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(alias = "type")]
    t_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Error::new(
            ErrorKind::Other,
            "please specify a single input file argument",
        ));
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(args[1].clone())?;

    for row in reader.deserialize() {
        let transaction: Transaction = row?;
        println!("{:?}", transaction);
    }

    Ok(())
}
