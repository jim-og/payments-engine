use std::io::{Error, ErrorKind};

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Amount(pub Decimal);

#[derive(Debug, Deserialize, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ClientId(u16);

#[derive(Debug, Deserialize)]
pub struct TransactionId(u32);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct TransactionEntry {
    #[serde(alias = "type")]
    pub transaction_type: TransactionType,
    pub client: ClientId,
    tx: TransactionId,
    pub amount: Option<Amount>,
}

pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

pub struct Deposit {
    client: ClientId,
    tx: TransactionId,
    amount: Amount,
}

pub struct Withdrawal {
    client: ClientId,
    tx: TransactionId,
    amount: Amount,
}

pub struct Dispute {
    client: ClientId,
    tx: TransactionId,
}

pub struct Resolve {
    client: ClientId,
    tx: TransactionId,
}

pub struct Chargeback {
    client: ClientId,
    tx: TransactionId,
}

impl TryFrom<TransactionEntry> for Transaction {
    type Error = std::io::Error;

    fn try_from(entry: TransactionEntry) -> Result<Self, std::io::Error> {
        let transaction = match entry.transaction_type {
            TransactionType::Deposit => Transaction::Deposit(Deposit {
                client: entry.client,
                tx: entry.tx,
                amount: entry.amount.ok_or_else(|| {
                    Error::new(ErrorKind::Other, "desposit does not contain an amount")
                })?,
            }),
            TransactionType::Withdrawal => Transaction::Withdrawal(Withdrawal {
                client: entry.client,
                tx: entry.tx,
                amount: entry.amount.ok_or_else(|| {
                    Error::new(ErrorKind::Other, "withdrawal does not contain an amount")
                })?,
            }),
            TransactionType::Dispute => {
                if entry.amount.is_some() {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "dispute contains unexpected amount",
                    ));
                }
                Transaction::Dispute(Dispute {
                    client: entry.client,
                    tx: entry.tx,
                })
            }
            TransactionType::Resolve => {
                if entry.amount.is_some() {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "resolve contains unexpected amount",
                    ));
                }
                Transaction::Resolve(Resolve {
                    client: entry.client,
                    tx: entry.tx,
                })
            }
            TransactionType::Chargeback => {
                if entry.amount.is_some() {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "chargeback contains unexpected amount",
                    ));
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
