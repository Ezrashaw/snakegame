#!/bin/bash
set -xe
cd "$(dirname "$0")/.."

rm -rf contrib/snake
mkdir contrib/snake

# Copy in setup script.
cp contrib/setup.sh contrib/snake/setup.sh
chmod +x contrib/snake/setup.sh

# Build main snake package.
cargo clean
RUSTFLAGS="-Ctarget-cpu=native -Clink-args=-Wl,-build-id=none,--no-eh-frame-hdr -Crelocation-model=static" cargo build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-musl --release
objcopy -R .eh_frame -R .got.plt -R .comment target/x86_64-unknown-linux-musl/release/snake contrib/snake/snake

# Create patched 16x32 consolefont.
cd crates/psf-util
cargo r
sudo setfont -d patched8x16.psfu -C /dev/tty63
sudo setfont -O patched16x32.psfu -C /dev/tty63
rm patched8x16.psfu
mv patched16x32.psfu ../../contrib/snake
cd -