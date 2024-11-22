use crate::types::{Amount, ClientId, Transaction, TransactionId};
use std::collections::{HashMap, HashSet};

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
    fn deposit(&self, amount: Amount) {
        // A deposit is a credit to the client's asset account, meaning it should increase the available and
        // total funds of the client account.
        todo!()
    }

    fn withdrawal(&self, amount: Amount) -> Result<(), std::io::Error> {
        // A withdraw is a debit to the client's asset account, meaning it should decrease the available and
        // total funds of the client account.

        // If a client does not have sufficient available funds the withdrawal should fail and the total amount
        // of funds should not change
        todo!()
    }

    fn dispute(&self, amount: Amount) -> Result<(), std::io::Error> {
        // A dispute represents a client's claim that a transaction was erroneous and should be reversed.
        // The transaction shouldn't be reversed yet but the associated funds should be held. This means
        // that the clients available funds should decrease by the amount disputed, their held funds should
        // increase by the amount disputed, while their total funds should remain the same.
        todo!()
    }

    fn resolve(&self, amount: Amount) -> Result<(), std::io::Error> {
        // A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
        // were previously disputed are no longer disputed. This means that the clients held funds should
        // decrease by the amount no longer disputed, their available funds should increase by the amount
        // no longer disputed, and their total funds should remain the same.
        todo!()
    }

    fn chargeback(&self, amount: Amount) -> Result<(), std::io::Error> {
        // A chargeback is the final state of a dispute and represents the client reversing a transaction.
        // Funds that were held have now been withdrawn. This means that the clients held funds and total
        // funds should decrease by the amount previously disputed. If a chargeback occurs the client's
        // account should be immediately frozen.
        todo!()
    }
}

pub struct Ledger {
    clients: HashMap<ClientId, Account>,
    transactions: HashMap<(ClientId, TransactionId), Transaction>,
    disputes: HashSet<(ClientId, TransactionId)>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            clients: HashMap::new(),
            transactions: HashMap::new(),
            disputes: HashSet::new(),
        }
    }
}

impl Ledger {
    pub fn update(&mut self, transaction: Transaction) -> Result<(), std::io::Error> {
        match transaction {
            Transaction::Deposit(deposit) => todo!(),
            Transaction::Withdrawal(withdrawal) => todo!(),
            Transaction::Dispute(dispute) => todo!(),
            Transaction::Resolve(resolve) => todo!(),
            Transaction::Chargeback(chargeback) => todo!(),
        }
    }
}
