use crate::types::{LedgerEntry, ParseError, Transaction, TransactionEntry};
use std::io;

/// TODO
pub fn read_input(rdr: impl io::Read) -> impl Iterator<Item = Result<Transaction, ParseError>> {
    let reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) // Automatically trim whitespace
        .from_reader(rdr);
    reader.into_deserialize::<TransactionEntry>().map(|entry| {
        // Map potential csv::Error then convert into a Transaction
        let transaction_entry = entry.map_err(ParseError::Csv)?;
        Transaction::try_from(transaction_entry)
    })
}

/// TODO
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
}
