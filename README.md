# `onomy-rs` proof-of-concept for Equity/Zeno on Byzantine Reliable Broadcast

## Crates
### equity_types
A library crate of common definitions of many structs and traits

Note to developers: always explicitly specify the types going into and out of the databases, because
the generic serialization impls like to do stuff like elide to `()`.

### equity_storage
Wraps around in-memory and external databases

### equity_core
Contains the main `EquityService` running binary. e.x. `cargo r --bin equity_core`

### equity_client
Contains both a library and command-line interface binary for an `EquityClient` to interface with
`equity_core`. e.x. `cargo r --bin equity_client -- health` when equity_core is running

### testcrate
Contains all integration tests. The automated runner can be run with `cargo run --bin run_test --`
and arguments after the "--" are passed to the `run_test` binary. The `run_test` binary will start
up a network according to the `<TEST_MODE>`, and will further pass its args to the `test_runner`(s)
which will run in the network. The test runners run with only access to their binaries and an
`--internal` network. Logs for each of the containers will be streamed to files under
`testcrate/logs/`, or alternatively you can pass `--ci` and see them all inline.

### scripts
`./testcrate/scripts/fmt.sh` needs to ran before each Pull Request.

TODO we need to fix the error system, we probably need a crate just for errors and import handling

