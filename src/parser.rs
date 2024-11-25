use crate::types::{
    Amount, Chargeback, ClientId, Deposit, Dispute, Resolve, Transaction, TransactionId,
    TransactionType, Withdrawal,
};
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("deposit is missing an amount")]
    DepositMissing,
    #[error("withdrawal is missing an amount")]
    WithdrawalMissing,
    #[error("dispute contains unexpected amount")]
    DisputeUnexpected,
    #[error("resolve contains unexpected amount")]
    ResolveUnexpected,
    #[error("chargeback contains unexpected amount")]
    ChargebackUnexpected,
    #[error("error reading csv")]
    Csv(#[from] csv::Error),
}

#[derive(Debug, Deserialize)]
pub struct TransactionEntry {
    #[serde(alias = "type")]
    pub transaction_type: TransactionType,
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Option<Amount>,
}

impl TryFrom<TransactionEntry> for Transaction {
    type Error = ParseError;

    fn try_from(entry: TransactionEntry) -> Result<Self, ParseError> {
        let transaction = match entry.transaction_type {
            TransactionType::Deposit => Transaction::Deposit(Deposit {
                client: entry.client,
                tx: entry.tx,
                amount: entry.amount.ok_or(ParseError::DepositMissing)?,
            }),
            TransactionType::Withdrawal => Transaction::Withdrawal(Withdrawal {
                client: entry.client,
                tx: entry.tx,
                amount: entry.amount.ok_or(ParseError::WithdrawalMissing)?,
            }),
            TransactionType::Dispute => {
                if entry.amount.is_some() {
                    return Err(ParseError::DisputeUnexpected);
                }
                Transaction::Dispute(Dispute {
                    client: entry.client,
                    tx: entry.tx,
                })
            }
            TransactionType::Resolve => {
                if entry.amount.is_some() {
                    return Err(ParseError::ResolveUnexpected);
                }
                Transaction::Resolve(Resolve {
                    client: entry.client,
                    tx: entry.tx,
                })
            }
            TransactionType::Chargeback => {
                if entry.amount.is_some() {
                    return Err(ParseError::ChargebackUnexpected);
                }
                Transaction::Chargeback(Chargeback {
                    client: entry.client,
                    tx: entry.tx,
                })
            }
        };
        Ok(transaction)
    }
}

#[derive(Debug, Serialize)]
pub struct LedgerEntry {
    pub client: ClientId,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

/// Reads and parses data from a CSV input, returning an iterator of `Transaction` results.
/// This allows streaming of CSV data without loading the entire file into memory.
pub fn read_input(rdr: impl io::Read) -> impl Iterator<Item = Result<Transaction, ParseError>> {
    let reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(rdr);
    reader.into_deserialize::<TransactionEntry>().map(|entry| {
        // Map potential csv::Error then convert into a Transaction
        let transaction_entry = entry.map_err(ParseError::Csv)?;
        Transaction::try_from(transaction_entry)
    })
}

/// Write a sequence of `LedgerEntry` records to a CSV output.
pub fn write_output(
    wtr: impl io::Write,
    iter: impl Iterator<Item = LedgerEntry>,
) -> Result<(), std::io::Error> {
    let mut writer = csv::Writer::from_writer(wtr);
    for entry in iter {
        writer.serialize(entry)?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ledger::Account,
        types::{
            Amount, Chargeback, ClientId, Deposit, Dispute, Resolve, TransactionId, Withdrawal,
        },
    };
    use rust_decimal::Decimal;

    #[test]
    fn read_transactions() {
        let rdr = "type, client, tx, amount\n
                        deposit, 1, 1, 1.4567\n
                        withdrawal, 1, 4, 1.1864\n
                        dispute, 1, 1,\n
                        resolve, 2, 3,\n
                        chargeback, 2, 2,\n"
            .as_bytes();

        let got = read_input(rdr)
            .map(|transaction| transaction.unwrap())
            .collect::<Vec<_>>();

        let want = [
            Transaction::Deposit(Deposit {
                client: ClientId(1),
                tx: TransactionId(1),
                amount: Amount::from(Decimal::new(14567, 4)),
            }),
            Transaction::Withdrawal(Withdrawal {
                client: ClientId(1),
                tx: TransactionId(4),
                amount: Amount::from(Decimal::new(11864, 4)),
            }),
            Transaction::Dispute(Dispute {
                client: ClientId(1),
                tx: TransactionId(1),
            }),
            Transaction::Resolve(Resolve {
                client: ClientId(2),
                tx: TransactionId(3),
            }),
            Transaction::Chargeback(Chargeback {
                client: ClientId(2),
                tx: TransactionId(2),
            }),
        ];

        assert_eq!(got, want);
    }

    #[test]
    fn write_accounts() {
        use std::io::Cursor;
        let accounts = [
            Account {
                client_id: ClientId(1),
                available: Amount::from(Decimal::new(16587, 4)),
                held: Amount::from(Decimal::new(47654, 4)),
                locked: false,
            },
            Account {
                client_id: ClientId(2),
                available: Amount::from(Decimal::new(63625, 4)),
                held: Amount::from(Decimal::new(94532, 4)),
                locked: true,
            },
        ];

        let entries = accounts.iter().map(|account| LedgerEntry::from(account));

        let mut buffer = Cursor::new(Vec::new());
        write_output(&mut buffer, entries.into_iter()).expect("Failed to write output");

        let got = String::from_utf8(buffer.into_inner()).expect("Invalid UTF-8");
        let want = "\
            client,available,held,total,locked\n\
            1,1.6587,4.7654,6.4241,false\n\
            2,6.3625,9.4532,15.8157,true\n";
        assert_eq!(got, want);
    }

    #[test]
    fn deposit() {
        assert!(Transaction::try_from(TransactionEntry {
            transaction_type: TransactionType::Deposit,
            client: ClientId(1),
            tx: TransactionId(1),
            amount: Some(Amount::from(1)),
        })
        .is_ok());
    }

    #[test]
    fn deposit_missing_amount() {
        assert!(matches!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Deposit,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: None,
            }),
            Err(ParseError::DepositMissing)
        ));
    }

    #[test]
    fn withdrawal() {
        assert!(Transaction::try_from(TransactionEntry {
            transaction_type: TransactionType::Withdrawal,
            client: ClientId(1),
            tx: TransactionId(1),
            amount: Some(Amount::from(1)),
        })
        .is_ok());
    }

    #[test]
    fn withdrawal_missing_amount() {
        assert!(matches!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Withdrawal,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: None,
            }),
            Err(ParseError::WithdrawalMissing)
        ));
    }

    #[test]
    fn dispute() {
        assert!(Transaction::try_from(TransactionEntry {
            transaction_type: TransactionType::Dispute,
            client: ClientId(1),
            tx: TransactionId(1),
            amount: None,
        })
        .is_ok());
    }

    #[test]
    fn dispute_unexpected_amount() {
        assert!(matches!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Dispute,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: Some(Amount::from(1)),
            }),
            Err(ParseError::DisputeUnexpected)
        ));
    }

    #[test]
    fn resolve() {
        assert!(Transaction::try_from(TransactionEntry {
            transaction_type: TransactionType::Resolve,
            client: ClientId(1),
            tx: TransactionId(1),
            amount: None,
        })
        .is_ok());
    }

    #[test]
    fn resolve_unexpected_amount() {
        assert!(matches!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Resolve,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: Some(Amount::from(1)),
            }),
            Err(ParseError::ResolveUnexpected)
        ));
    }

    #[test]
    fn chargeback() {
        assert!(Transaction::try_from(TransactionEntry {
            transaction_type: TransactionType::Chargeback,
            client: ClientId(1),
            tx: TransactionId(1),
            amount: None,
        })
        .is_ok());
    }

    #[test]
    fn chargeback_unexpected_amount() {
        assert!(matches!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Chargeback,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: Some(Amount::from(1)),
            }),
            Err(ParseError::ChargebackUnexpected)
        ));
    }
}
