#!/bin/bash
# This is run in the beginning of each test to prepare everything

# https://stackoverflow.com/a/22644006
trap exit INT TERM
trap "kill 0" EXIT

set -e -x
mkdir -p data/
# Compile the programs
(cd ../testsignal; cargo build --release)
(cd ../spektri; cargo build --release)
# Add them to PATH
PATH="../testsignal/target/release/:../spektri/target/release/:../tools:$PATH"
