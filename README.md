# CZ4013 - Dsitributed flight information system

## Pre-requisite
+ Rust: [Installation guide](https://www.rust-lang.org/tools/install)

## Build
```bash
cargo build
```

## Code Structure

- `<root>/rpc_contracts` - Public contracts for client and server
  + `<root>/rpc_contracts/body` - defines interface body for request and response, also defines `EncodeBody` and `DecodeBody` traits to turn values into bytes, and vice versa.
- `<root>/client` - Client program, please see:
  + `<root>/server/src/api`: Client API service consumer implementations
- `<root>/server` - Server program, please see:
  + `<root>/server/database`: Database module implementations
  + `<root>/server/src/api`: Server API service handler implementations
- `<root>/serde` - Serialization/Deserialization facilities for `server` and `client` programs. Support simple (C-like) enum and named fields structs
  + `<root>/serde/src/ser` - generic layout of `Serialize` trait
  + `<root>/serde/src/de` - generic layout of `Deserialize` trait 
  + `<root>/serde/json` - JSON implementations for `Serialize` and `Deserialize` traits, inspired by [miniserde](https://github.com/dtolnay/miniserde).
  + `<root>/serde/serde_derive` - `derive` proc macro for `Serialize` and `Deserialize` traits, realizing JSON implementations on arbitrary supported types/structs/enum

## Starting
Executable formats
```bash
cargo run --bin <exe_regex>
```
For a full list of executables, please see `Cargo.toml` and `src/bin` of each module/sub-module.

### Seed Database
```bash
cd server
cargo run --bin seed_db
```


### Client
```bash
cargo run --bin client
```
DEBUG mode, require [cargo-watch](https://crates.io/crates/cargo-watch)
```bash
cargo watch -x 'run --bin client'
```

### Server
```bash
cargo run --bin server
```
DEBUG mode, require [cargo-watch](https://crates.io/crates/cargo-watch)
```bash
cargo watch -x 'run --bin server'
```

## Testing
Only unit tests for `database` services are available at the moment

To test (from `<root>`)
```bash
cd server/database
cargo test
```

Execute a single test:
```bash
cargo test <test_name_regex>
```

More about [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html) 
