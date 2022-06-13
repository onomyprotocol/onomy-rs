# `onomy-rs` proof-of-concept for Equity/Zeno on Byzantine Reliable Broadcast

## Crates
### equity-types
A library crate of common definitions of many structs and traits

### equity-storage
Wraps around in-memory and external databases

### equity-core
Contains the main `EquityService` running binary

### equity-client
Contains both a library and command-line interface binary for an `EquityClient` to interface with
`equity-core`

### testcrate
Contains all integration tests.
