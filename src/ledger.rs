use crate::types::{Amount, ClientId, Transaction};
use std::collections::HashMap;

struct Account {
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            available: Amount(0.into()),
            held: Amount(0.into()),
            locked: false,
        }
    }
}

impl Account {
    fn update(&self, transaction: Transaction) -> Result<(), std::io::Error> {
        match transaction.transaction_type {
            crate::types::TransactionType::Deposit => todo!(),
            crate::types::TransactionType::Withdrawal => todo!(),
            crate::types::TransactionType::Dispute => todo!(),
            crate::types::TransactionType::Resolve => todo!(),
            crate::types::TransactionType::Chargeback => todo!(),
        }
    }
}

pub struct Ledger {
    clients: HashMap<ClientId, Account>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
}

impl Ledger {
    pub fn update(&self, transaction: Transaction) -> Result<(), std::io::Error> {
        println!("{:?}", transaction);
        match transaction.transaction_type {
            crate::types::TransactionType::Deposit => todo!(),
            crate::types::TransactionType::Withdrawal => todo!(),
            crate::types::TransactionType::Dispute => todo!(),
            crate::types::TransactionType::Resolve => todo!(),
            crate::types::TransactionType::Chargeback => todo!(),
        }
    }
}
