use crate::types::{
    Amount, ClientId, Deposit, Dispute, Resolve, Transaction, TransactionId, Withdrawal,
};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PaymentError {
    #[error("client {client_id:?} has insufficient funds to withdraw {requested:?} (available {available:?})")]
    WithdrawalInsufficientFunds {
        client_id: ClientId,
        available: Amount,
        requested: Amount,
    },
    #[error("failed to dispute transaction, transaction id {transaction_id:?} does not exist for client {client_id:?})")]
    DisputeFailed {
        client_id: ClientId,
        transaction_id: TransactionId,
    },
    #[error("failed to resolve dispute, transaction id {transaction_id:?} is not under dispute for client {client_id:?})")]
    ResolveFailed {
        client_id: ClientId,
        transaction_id: TransactionId,
    },
    #[error("failed to chargeback dispute, transaction id {transaction_id:?} is not under dispute for client {client_id:?})")]
    ChargebackFailed {
        client_id: ClientId,
        transaction_id: TransactionId,
    },
    #[error("account has been locked for client {client_id:?}, operation failed)")]
    ClientAccountLocked { client_id: ClientId },
}

struct Account {
    client_id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Account {
    fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            available: Amount::from(0),
            held: Amount::from(0),
            locked: false,
        }
    }

    /// A deposit is a credit to the client's asset account, meaning it should increase the available and
    /// total funds of the client account.
    fn deposit(&mut self, amount: Amount) -> Result<(), PaymentError> {
        if self.locked {
            return Err(PaymentError::ClientAccountLocked {
                client_id: self.client_id,
            });
        }
        self.available.0 += amount.0;
        Ok(())
    }

    /// A withdraw is a debit to the client's asset account, meaning it should decrease the available and
    /// total funds of the client account. If a client does not have sufficient available funds the withdrawal
    /// should fail and the total amount of funds should not change.
    fn withdrawal(&mut self, amount: Amount) -> Result<(), PaymentError> {
        if self.locked {
            return Err(PaymentError::ClientAccountLocked {
                client_id: self.client_id,
            });
        }
        if self.available.0 >= amount.0 {
            self.available.0 -= amount.0;
            Ok(())
        } else {
            Err(PaymentError::WithdrawalInsufficientFunds {
                client_id: self.client_id,
                available: self.available,
                requested: amount,
            })
        }
    }

    /// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
    /// The transaction shouldn't be reversed yet but the associated funds should be held. This means
    /// that the clients available funds should decrease by the amount disputed, their held funds should
    /// increase by the amount disputed, while their total funds should remain the same.
    fn dispute(&mut self, amount: Amount) -> Result<(), PaymentError> {
        if self.locked {
            return Err(PaymentError::ClientAccountLocked {
                client_id: self.client_id,
            });
        }

        // It is assumed that a client always has a sufficient available funds for the amount disputed
        // to be held.
        self.available.0 -= amount.0;
        self.held.0 += amount.0;
        Ok(())
    }

    /// A resolve represents a resolution to a dispute, releasing the associated held funds. Funds that
    /// were previously disputed are no longer disputed. This means that the clients held funds should
    /// decrease by the amount no longer disputed, their available funds should increase by the amount
    /// no longer disputed, and their total funds should remain the same.
    fn resolve(&mut self, amount: Amount) -> Result<(), PaymentError> {
        self.available.0 += amount.0;
        self.held.0 -= amount.0;
        Ok(())
    }

    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// Funds that were held have now been withdrawn. This means that the clients held funds and total
    /// funds should decrease by the amount previously disputed. If a chargeback occurs the client's
    /// account should be immediately frozen.
    fn chargeback(&self, amount: Amount) -> Result<(), PaymentError> {
        todo!()
    }
}

pub struct Ledger {
    clients: HashMap<ClientId, Account>,
    deposits: HashMap<(ClientId, TransactionId), Amount>,
    disputes: HashSet<(ClientId, TransactionId)>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            clients: HashMap::new(),
            deposits: HashMap::new(),
            disputes: HashSet::new(),
        }
    }
}

impl Ledger {
    pub fn update(&mut self, transaction: Transaction) -> Result<(), PaymentError> {
        match transaction {
            Transaction::Deposit(Deposit { client, tx, amount }) => {
                self.clients
                    .entry(client)
                    .or_insert(Account::new(client))
                    .deposit(amount)?;
                self.deposits.insert((client, tx), amount);
            }
            Transaction::Withdrawal(Withdrawal {
                client,
                tx: _,
                amount,
            }) => {
                self.clients
                    .entry(client)
                    .or_insert(Account::new(client))
                    .withdrawal(amount)?;
            }
            Transaction::Dispute(Dispute { client, tx }) => {
                // Find the transaction amount
                let amount = match self.deposits.get(&(client, tx)) {
                    Some(deposit) => deposit,
                    None => {
                        return Err(PaymentError::DisputeFailed {
                            client_id: client,
                            transaction_id: tx,
                        })
                    }
                };

                // Update the client's account
                self.clients
                    .entry(client)
                    .or_insert(Account::new(client))
                    .dispute(*amount)?;

                // Track the dispute
                self.disputes.insert((client, tx));
            }
            Transaction::Resolve(Resolve { client, tx }) => {
                // Confirm a dispute exists
                self.disputes
                    .get(&(client, tx))
                    .ok_or_else(|| PaymentError::ResolveFailed {
                        client_id: client,
                        transaction_id: tx,
                    })?;

                // Find the transaction amount
                let amount = match self.deposits.get(&(client, tx)) {
                    Some(deposit) => deposit,
                    None => {
                        return Err(PaymentError::ResolveFailed {
                            client_id: client,
                            transaction_id: tx,
                        })
                    }
                };

                // Update the client's account
                self.clients
                    .entry(client)
                    .or_insert(Account::new(client))
                    .resolve(*amount)?;

                // Clear the dispute
                self.disputes.remove(&(client, tx));
            }
            Transaction::Chargeback(chargeback) => todo!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};

    #[test]
    fn deposit() {
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let transaction_id = TransactionId(1);
        let amount = Amount(7.into());

        let transaction = Transaction::Deposit(Deposit {
            client: client_id,
            tx: transaction_id,
            amount: amount.clone(),
        });

        ledger.update(transaction).unwrap();

        // Assert client account has been created and the deposit added.
        let Account {
            available,
            held,
            locked,
            ..
        } = ledger
            .clients
            .get(&client_id)
            .expect("client not found in ledger");

        assert_eq!(
            (available, held, locked),
            (&amount, &Amount(0.into()), &false)
        );

        // Assert that the transaction has been stored.
        assert!(ledger.deposits.contains_key(&(client_id, transaction_id)));

        // Assert that there are no disputes
        assert!(ledger.disputes.is_empty());
    }

    #[test]
    fn withdrawal() {
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id = TransactionId(1);
        let withdrawal_id = TransactionId(2);

        let transactions = [
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id,
                amount: Amount::from(7),
            }),
            Transaction::Withdrawal(Withdrawal {
                client: client_id,
                tx: withdrawal_id,
                amount: Amount::from(3),
            }),
        ];

        transactions
            .into_iter()
            .for_each(|transaction| ledger.update(transaction).unwrap());

        // Assert client account has been created, the deposit added, and withdrawal made.
        let Account {
            available,
            held,
            locked,
            ..
        } = ledger
            .clients
            .get(&client_id)
            .expect("client not found in ledger");

        assert_eq!(
            (available, held, locked),
            (&Amount::from(4), &Amount::from(0), &false)
        );

        // Assert that the deposit has been stored.
        assert!(ledger.deposits.contains_key(&(client_id, deposit_id)));

        // Assert that there are no disputes
        assert!(ledger.disputes.is_empty());
    }

    #[test]
    fn withdrawal_insufficient_funds() {
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id = TransactionId(1);
        let deposit_amount = Amount::from(5);
        let withdrawal_id = TransactionId(2);
        let withdrawal_amount = Amount::from(7);

        ledger
            .update(Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id,
                amount: deposit_amount.clone(),
            }))
            .unwrap();

        let withdrawal_result = ledger.update(Transaction::Withdrawal(Withdrawal {
            client: client_id,
            tx: withdrawal_id,
            amount: withdrawal_amount.clone(),
        }));

        // Assert that the withdrawal failed
        assert_eq!(
            withdrawal_result,
            Err(PaymentError::WithdrawalInsufficientFunds {
                client_id,
                available: deposit_amount.clone(),
                requested: withdrawal_amount
            })
        );

        // Assert client account has been created, the deposit added, and that the withdrawal failed.
        let Account {
            available,
            held,
            locked,
            ..
        } = ledger
            .clients
            .get(&client_id)
            .expect("client not found in ledger");

        assert_eq!(
            (available, held, locked),
            (&deposit_amount, &Amount::from(0), &false)
        );

        // Assert that the deposit has been stored.
        assert!(ledger.deposits.contains_key(&(client_id, deposit_id)));

        // Assert that there are no disputes
        assert!(ledger.disputes.is_empty());
    }

    #[test]
    fn dispute() {
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id_1 = TransactionId(1);
        let deposit_id_2 = TransactionId(2);
        let amount_available = Amount::from(5);
        let amount_held = Amount::from(3);

        let transactions = [
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_1,
                amount: amount_available.clone(),
            }),
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_2,
                amount: amount_held.clone(),
            }),
            Transaction::Dispute(Dispute {
                client: client_id,
                tx: deposit_id_2,
            }),
        ];

        transactions
            .into_iter()
            .for_each(|transaction| ledger.update(transaction).unwrap());

        // Assert that the client account has been created, the deposits added, and the correct amount is held in dispute.
        let Account {
            available,
            held,
            locked,
            ..
        } = ledger
            .clients
            .get(&client_id)
            .expect("client not found in ledger");

        assert_eq!(
            (available, held, locked),
            (&amount_available, &amount_held, &false)
        );

        // Assert that the dispute is being tracked
        assert!(ledger.disputes.contains(&(client_id, deposit_id_2)));
    }

    #[test]
    fn dispute_no_transaction() {
        // If the tx specified by the dispute doesn't exist this is an error on the partner's side.
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id = TransactionId(1);
        let deposit_amount = Amount::from(7);
        let dispute_id = TransactionId(2);

        ledger
            .update(Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id,
                amount: deposit_amount.clone(),
            }))
            .unwrap();

        let dispute_result = ledger.update(Transaction::Dispute(Dispute {
            client: client_id,
            tx: dispute_id,
        }));

        // Assert that the dispute failed
        assert_eq!(
            dispute_result,
            Err(PaymentError::DisputeFailed {
                client_id: client_id,
                transaction_id: dispute_id
            })
        );

        // Assert client account has been created, the deposit added, and that no funds are held.
        let Account {
            available,
            held,
            locked,
            ..
        } = ledger
            .clients
            .get(&client_id)
            .expect("client not found in ledger");

        assert_eq!(
            (available, held, locked),
            (&deposit_amount, &Amount::from(0), &false)
        );

        // Assert that there are no disputes
        assert!(ledger.disputes.is_empty());
    }

    #[test]
    fn resolve() {
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id_1 = TransactionId(1);
        let deposit_id_2 = TransactionId(2);
        let amount_1 = Amount::from(5);
        let amount_2 = Amount::from(3);

        let transactions = [
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_1,
                amount: amount_1.clone(),
            }),
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_2,
                amount: amount_2.clone(),
            }),
            Transaction::Dispute(Dispute {
                client: client_id,
                tx: deposit_id_2,
            }),
            Transaction::Resolve(Resolve {
                client: client_id,
                tx: deposit_id_2,
            }),
        ];

        transactions
            .into_iter()
            .for_each(|transaction| ledger.update(transaction).unwrap());

        // Assert that the client account has been created, the deposits added, and that the dispute has been resolved.
        let Account {
            available,
            held,
            locked,
            ..
        } = ledger
            .clients
            .get(&client_id)
            .expect("client not found in ledger");

        assert_eq!(
            (available, held, locked),
            (
                &Amount::from(amount_1.0 + amount_2.0),
                &Amount::from(0),
                &false
            )
        );

        // Assert that there are no disputes
        assert!(ledger.disputes.is_empty());
    }

    #[test]
    fn resolve_transaction_not_disputed() {
        // If the tx specified by the resolve isn't under dispute this is an error on the partner's side.
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id_1 = TransactionId(1);
        let deposit_id_2 = TransactionId(2);
        let amount_1 = Amount::from(5);

        ledger
            .update(Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_1,
                amount: amount_1.clone(),
            }))
            .unwrap();

        let resolve_result = ledger.update(Transaction::Resolve(Resolve {
            client: client_id,
            tx: deposit_id_2,
        }));

        // Assert that the resolve failed
        assert_eq!(
            resolve_result,
            Err(PaymentError::ResolveFailed {
                client_id: client_id,
                transaction_id: deposit_id_2
            })
        );
    }

    #[test]
    fn chargeback() {
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id_1 = TransactionId(1);
        let deposit_id_2 = TransactionId(2);
        let amount_1 = Amount::from(5);
        let amount_2 = Amount::from(3);

        let transactions = [
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_1,
                amount: amount_1.clone(),
            }),
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_2,
                amount: amount_2.clone(),
            }),
            Transaction::Dispute(Dispute {
                client: client_id,
                tx: deposit_id_2,
            }),
            Transaction::Chargeback(Chargeback {
                client: client_id,
                tx: deposit_id_2,
            }),
        ];

        transactions
            .into_iter()
            .for_each(|transaction| ledger.update(transaction).unwrap());

        // Assert that the client account has been created, the deposits added, the dispute has been charged back,
        // and that the client has been locked
        let Account {
            available,
            held,
            locked,
            ..
        } = ledger
            .clients
            .get(&client_id)
            .expect("client not found in ledger");

        assert_eq!(
            (available, held, locked),
            (&amount_1, &Amount::from(0), &true)
        );

        // Assert that there are no disputes
        assert!(ledger.disputes.is_empty());
    }

    #[test]
    fn chargeback_transaction_not_disputed() {
        // If the tx specified by the chargeback isn't under dispute this is an error on the partner's side.
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id_1 = TransactionId(1);
        let deposit_id_2 = TransactionId(2);
        let amount_1 = Amount::from(5);

        ledger
            .update(Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_1,
                amount: amount_1.clone(),
            }))
            .unwrap();

        let resolve_result = ledger.update(Transaction::Chargeback(Chargeback {
            client: client_id,
            tx: deposit_id_2,
        }));

        // Assert that the resolve failed
        assert_eq!(
            resolve_result,
            Err(PaymentError::ChargebackFailed {
                client_id: client_id,
                transaction_id: deposit_id_2
            })
        );
    }

    #[test]
    fn client_account_locked() {
        // Validate that once a client account is locked, all further transactions fail.
        let mut ledger = Ledger::default();
        let client_id = ClientId(1);
        let deposit_id_1 = TransactionId(1);

        let transactions = [
            Transaction::Deposit(Deposit {
                client: client_id,
                tx: deposit_id_1,
                amount: Amount::from(5),
            }),
            Transaction::Dispute(Dispute {
                client: client_id,
                tx: deposit_id_1,
            }),
            Transaction::Chargeback(Chargeback {
                client: client_id,
                tx: deposit_id_1,
            }),
        ];

        transactions
            .into_iter()
            .for_each(|transaction| ledger.update(transaction).unwrap());

        // Assert deposit fails
        assert_eq!(
            ledger.update(Transaction::Deposit(Deposit {
                client: client_id,
                tx: TransactionId(2),
                amount: Amount::from(3),
            })),
            Err(PaymentError::ClientAccountLocked { client_id })
        );

        // Assert withdrawal fails
        assert_eq!(
            ledger.update(Transaction::Withdrawal(Withdrawal {
                client: client_id,
                tx: TransactionId(3),
                amount: Amount::from(3),
            })),
            Err(PaymentError::ClientAccountLocked { client_id })
        );

        // Assert dispute fails
        assert_eq!(
            ledger.update(Transaction::Dispute(Dispute {
                client: client_id,
                tx: TransactionId(1),
            })),
            Err(PaymentError::ClientAccountLocked { client_id })
        );
        assert!(ledger.disputes.is_empty());
    }
}
