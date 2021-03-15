#!/bin/bash
(cd testsignal; cargo build --release) &&
(cd spektri; cargo build --release) &&
time testsignal/target/release/testsignal complex 100000000 | (time spektri/target/release/spektri)
