#[cfg(test)]
mod tests {
    use payments_engine::ledger::Ledger;
    use std::io::Cursor;

    #[test]
    fn end_to_end() {
        let rdr = "type, client, tx, amount\n
            deposit,	1,	1,	0.2436\n
            deposit,	1,	2,	0.3965\n
            deposit,	1,	3,	0.0027\n
            withdrawal,	1,	4,	0.1374\n
            dispute,	1,	1,	\n
            deposit,	2,	5,	0.8263\n
            deposit,	2,	6,	1.2749\n
            withdrawal,	2,	7,	0.0537\n
            dispute,	2,	5,	\n
            chargeback,	2,	5,	\n"
            .as_bytes();

        // Create a ledger to track client transactions.
        let mut ledger = Ledger::default();

        // Load transactions into the ledger.
        ledger.load(rdr);

        // Output client accounts
        let mut wrt = Cursor::new(Vec::new());
        ledger.print(&mut wrt).unwrap();
        let got = String::from_utf8(wrt.into_inner()).expect("Invalid UTF-8");

        // Not guaranteed which order client accounts will be in.
        let want_options = [
            "\
            client,available,held,total,locked\n\
            2,1.2212,0.0000,1.2212,true\n\
            1,0.2618,0.2436,0.5054,false\n"
                .to_string(),
            "\
            client,available,held,total,locked\n\
            1,0.2618,0.2436,0.5054,false\n\
            2,1.2212,0.0000,1.2212,true\n"
                .to_string(),
        ];

        assert!(want_options.contains(&got));
    }
}
