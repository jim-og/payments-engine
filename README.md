# Payments Engine

## Usage

```
$ cargo run -- transactions.csv > accounts.csv
```

To run tests:
```
$ cargo test
```

## Assumptions

- Only `deposit` transactions can be disputed.
- Once a client's account is locked all subsequent transactions performed on it will fail.
- If a client does not exist only a `deposit` transaction can create it.
- Malformed input lines, such as a `dispute` transaction which contains an amount, will be rejected.  

## Design

### parser.rs
Responsible for reading from an input and writing to an output in CSV format.
- CSV data is streamed in without loading the entire file into memory. 
- Input CSV data is deserialized into an internal data representation of a `Transaction` using `serde`. Input validation is performed to ensure each transaction is well formed.
- Output CSV data is serialized from a client's account using `serde`.

### ledger.rs
Responsible for maintaining a ledger of client accounts and the state of transaction disputes. The following data stores are maintained: 
- *clients* - holds each client’s account information of available funds,held funds, and locked status.
- *deposits* - tracks all the deposit transactions which have been made for all clients. This allows O(1) lookup time of a deposit in the event a transaction is disputed.
- *disputes* - tracks any active disputes. 

### types.rs
Used to specify types used by both parser and ledger.
- `rust_decimal` is used to ensure a precision of 4 decimal places is maintained when handling transaction amounts.

## Improvements

- The current design has synchronous reading of input and update of the ledger. This could be modified to work asynchronously, allowing CSVs from thousands of TCP streams to be processed concurrently. A summary of the changes required to achieve this:
    - Use `Tokio` for asynchronous IO.
    - Modify how the input files are read to return an async iterator.
    - `Mutex` the ledger's data stores to ensure shared mutable state is thread safe.
    - Modify the ledger's update methods to be `async` and have load `await` the responses.

- The choice to store all deposits in a separate datastore has simplified insertion and retrieval of past deposits, ensuring that disputes and resolves can be executed in O(1) time. However, storing these separate to the client’s account alongside deposits made by other clients may be suboptimal for other use cases. An example would be if we wanted to print the historic deposits made by a specific client. The current set up would require iterating through every deposit in the ledger to collect deposits made by this client.

- Storing deposits from every client in a single data store may be suboptimal. An example could be if we wanted to add a feature which printed all historic deposits made by a client. Our current setup would require us to iterate through every transaction processed by the ledger which would be very inefficient. A better solution would be for each client to have their own deposit data store.

- There should be a more robust set of integration tests with a large input CSV representating the amount of transactions the engine is expected to process.
