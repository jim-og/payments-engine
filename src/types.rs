use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
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

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, Hash, PartialEq)]
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
