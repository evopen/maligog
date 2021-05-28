#!/bin/bash

export RUSTFLAGS="-Zinstrument-coverage"
rm *.profraw
rm -rf ./target/debug/coverage/
cargo build
LLVM_PROFILE_FILE="%p-%m.profraw" cargo test --verbose
grcov . --binary-path ./target/debug/ -s . -t html --branch --llvm --ignore-not-existing --ignore "/*" -o ./target/debug/coverage/

xdg-open target/debug/coverage/index.html
