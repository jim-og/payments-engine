use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
pub struct Amount(pub Decimal);

impl From<i32> for Amount {
    fn from(value: i32) -> Self {
        Amount(Decimal::from(value))
    }
}

impl From<Decimal> for Amount {
    fn from(value: Decimal) -> Self {
        Amount(value)
    }
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ClientId(pub u16);

#[derive(Debug, Deserialize, Copy, Clone, Eq, Hash, PartialEq)]
pub struct TransactionId(pub u32);

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

#[derive(Debug, PartialEq)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

#[derive(Debug, PartialEq)]

pub struct Deposit {
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Amount,
}

#[derive(Debug, PartialEq)]
pub struct Withdrawal {
    pub client: ClientId,
    #[allow(dead_code)]
    pub tx: TransactionId,
    pub amount: Amount,
}

#[derive(Debug, PartialEq)]
pub struct Dispute {
    pub client: ClientId,
    pub tx: TransactionId,
}

#[derive(Debug, PartialEq)]
pub struct Resolve {
    pub client: ClientId,
    pub tx: TransactionId,
}

#[derive(Debug, PartialEq)]
pub struct Chargeback {
    pub client: ClientId,
    pub tx: TransactionId,
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Deposit,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: None,
            }),
            Err(ParseError::DepositMissing)
        );
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
        assert_eq!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Withdrawal,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: None,
            }),
            Err(ParseError::WithdrawalMissing)
        );
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
        assert_eq!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Dispute,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: Some(Amount::from(1)),
            }),
            Err(ParseError::DisputeUnexpected)
        );
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
        assert_eq!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Resolve,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: Some(Amount::from(1)),
            }),
            Err(ParseError::ResolveUnexpected)
        );
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
        assert_eq!(
            Transaction::try_from(TransactionEntry {
                transaction_type: TransactionType::Chargeback,
                client: ClientId(1),
                tx: TransactionId(1),
                amount: Some(Amount::from(1)),
            }),
            Err(ParseError::ChargebackUnexpected)
        );
    }
}
