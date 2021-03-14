#!/bin/sh
(cd testsignal; cargo build)
(cd spektri; cargo build)
testsignal/target/debug/testsignal complex 10000000 | spektri/target/debug/spektri
