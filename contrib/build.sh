#!/bin/bash
set -xe
cd "$(dirname "$0")/.."

rm -rf contrib/snake
mkdir contrib/snake

# Copy in setup script.
cp contrib/setup.sh contrib/snake/setup.sh
chmod +x contrib/snake/setup.sh

# Build main snake package and the server.
cargo clean
RUSTFLAGS="-Zlocation-detail=none -Ctarget-cpu=native -Clink-args=-Wl,-build-id=none,--no-eh-frame-hdr -Crelocation-model=static" cargo build \
    -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-musl --release --bin snake --bin snake-server --no-default-features
objcopy -R .eh_frame -R .got.plt -R .comment target/x86_64-unknown-linux-musl/release/snake contrib/snake/snake
objcopy -R .eh_frame -R .got.plt -R .comment target/x86_64-unknown-linux-musl/release/snake-server contrib/snake/snake-server

# Create patched 16x32 consolefont.
cargo r --bin psf-util
mv patched16x32.psfu contrib/snake
