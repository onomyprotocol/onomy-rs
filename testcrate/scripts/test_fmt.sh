#!/bin/bash
set -eux
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO=$DIR/../..
# for the CI
pushd $REPO && cargo sort -g -c
pushd $REPO/equity_client && cargo sort -g -c
pushd $REPO/equity_consensus && cargo sort -g -c
pushd $REPO/equity_core && cargo sort -g -c
pushd $REPO/equity_p2p && cargo sort -g -c
pushd $REPO/equity_storage && cargo sort -g -c
pushd $REPO/equity_types && cargo sort -g -c
pushd $REPO/testcrate && cargo sort -g -c
