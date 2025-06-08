#!/bin/sh
set -xe

# Tell the kernel not to write `prink` messages to the console.
echo 1 > /proc/sys/kernel/printk

# Load the patched console font.
setfont patched16x32.psfu
