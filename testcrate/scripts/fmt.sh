#!/bin/bash
set -eux
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO=$DIR/../..
pushd $REPO

# runs `clippy`, `fmt`, `cargo-sort` (you may need to run
# `cargo install cargo-sort`), and removes log files
cargo clippy --all --all-targets --all-features -- -D clippy::all
cargo +nightly fmt
pushd $REPO && cargo sort -g
pushd $REPO/equity_client && cargo sort -g
pushd $REPO/equity_consensus && cargo sort -g
pushd $REPO/equity_core && cargo sort -g
pushd $REPO/equity_p2p && cargo sort -g
pushd $REPO/equity_storage && cargo sort -g
pushd $REPO/equity_types && cargo sort -g
pushd $REPO/testcrate && cargo sort -g
pushd $REPO/testcrate/logs && rm -f *.log
