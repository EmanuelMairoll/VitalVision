#!/bin/bash

set -e

THISDIR=$(dirname $0)
cd $THISDIR

# Build the project for the desired platforms:
cargo build --target x86_64-apple-darwin
cargo build --target aarch64-apple-darwin
mkdir -p ./target/universal-macos/debug

lipo \
    ./target/aarch64-apple-darwin/debug/libvital_vision_core.a \
    ./target/x86_64-apple-darwin/debug/libvital_vision_core.a -create -output \
    ./target/universal-macos/debug/libvital_vision_core.a

cargo build --target aarch64-apple-ios

cargo build --target x86_64-apple-ios
cargo build --target aarch64-apple-ios-sim
mkdir -p ./target/universal-ios/debug

lipo \
    ./target/aarch64-apple-ios-sim/debug/libvital_vision_core.a \
    ./target/x86_64-apple-ios/debug/libvital_vision_core.a -create -output \
    ./target/universal-ios/debug/libvital_vision_core.a


#rm -r ./VitalVisionCore

swift-bridge-cli create-package \
  --bridges-dir ./generated \
  --out-dir VitalVisionCore \
  --ios target/aarch64-apple-ios/debug/libvital_vision_core.a \
  --simulator target/universal-ios/debug/libvital_vision_core.a \
  --macos target/universal-macos/debug/libvital_vision_core.a \
  --name VitalVisionCore