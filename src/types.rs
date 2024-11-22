use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Amount(pub Decimal);

#[derive(Debug, Deserialize)]
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
pub struct Transaction {
    #[serde(alias = "type")]
    pub transaction_type: TransactionType,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Amount>,
}
