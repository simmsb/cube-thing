#!/bin/bash
ssh cube "mv cube-thing cube-thing.old"
cross build --target armv7-unknown-linux-gnueabihf --release
scp target/armv7-unknown-linux-gnueabihf/release/cube-thing cube:cube-thing
